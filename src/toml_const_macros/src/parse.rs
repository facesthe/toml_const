//! Custom input syntax for proc-macro inputs

use std::fs;
use std::path::PathBuf;

use proc_macro2 as pm2;
use proc_macro2::{Delimiter, Group};
use quote::{ToTokens, TokenStreamExt, quote};
use syn::Ident;
use syn::{LitStr, braced, parse::Parse, punctuated::Punctuated};

/// Input to `toml_const` macro
///
/// ```ignore
/// // Point to a single file
/// toml_const!(pub TOML_CONST_STATIC: "some_file.toml");
///
/// // point to multiple files
/// // these files are checked in sequence for "use = true", and the first matching
/// // file is merged with the template file. If there are none, only the template file is used.
/// toml_const! {
///     pub TOML_CONST_STATIC: "some_template.toml" {
///         "some_substituion_1.toml";
///         "some_substituion_2.toml";
///     }
/// }
/// ```
#[derive(Clone)]
pub struct MacroInput {
    /// Whether the static variable is public
    pub is_pub: bool,

    /// Static item identifier
    pub item_ident: Ident,

    /// Path to the template file, mandatory
    pub path: LitStr,

    /// Any optional paths to substitute over the first path
    pub sub_paths: Option<Vec<LitStr>>,
}

impl Parse for MacroInput {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let is_pub: bool = {
            let lookahead = input.lookahead1();
            match lookahead.peek(syn::Token![pub]) {
                true => {
                    let _: syn::Token![pub] = input.parse()?;
                    true
                }
                false => false,
            }
        };

        let item_ident: syn::Ident = input.parse()?;
        let _: syn::Token![:] = input.parse()?;

        let template: LitStr = input.parse()?;

        let lookahead = input.lookahead1();

        let sub_paths = match lookahead.peek(syn::token::Brace) {
            true => {
                let content;
                braced!(content in input);

                let lit_str_vec =
                    Punctuated::<LitStr, syn::token::Semi>::parse_terminated(&content)?;

                let res = lit_str_vec.into_iter().map(|sp| sp).collect::<Vec<_>>();
                Some(res)
            }
            false => None,
        };

        Ok(Self {
            is_pub,
            item_ident,
            path: template,
            sub_paths,
        })
    }
}

impl ToTokens for MacroInput {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        if self.is_pub {
            quote! {pub}.to_tokens(tokens);
        }

        self.item_ident.to_tokens(tokens);
        quote! {:}.to_tokens(tokens);
        self.path.to_tokens(tokens);

        match &self.sub_paths {
            Some(sub) => {
                let subs = sub.iter().collect::<Punctuated<_, syn::Token![;]>>();

                tokens.append(Group::new(Delimiter::Brace, subs.to_token_stream()));
            }
            None => (),
        }
    }
}

impl MacroInput {
    /// Return one or more const definitions to an underscore expression (`_`).\
    ///
    /// These are calls to [include_str!] containing absolute paths.
    pub fn to_const_defs(&self, base_path: &PathBuf) -> pm2::TokenStream {
        let mut template_path = base_path.clone();
        template_path.push(PathBuf::from(&self.path.value()));
        let template_path = pathbuf_to_str(&template_path);

        let mut const_defs = vec![quote! {const _: &'static str = include_str!(#template_path);}];

        match &self.sub_paths {
            Some(sp) => {
                let additions = sp.iter().map(|sub_path| {
                    let mut abs_sub_path = base_path.clone();
                    abs_sub_path.push(PathBuf::from(sub_path.value()));
                    let sub_path = pathbuf_to_str(&abs_sub_path);

                    quote! {
                        const _: &'static str = include_str!(#sub_path);
                    }
                });

                const_defs.extend(additions);
            }
            None => (),
        }

        let collected = const_defs
            .into_iter()
            .map(|cd| cd)
            .collect::<pm2::TokenStream>();

        collected
    }

    /// Create a clone of `self` with all inner paths turned to absolute paths.
    ///
    /// The input base path must be absolute.
    pub fn to_abs_path(&self, base_path: &PathBuf) -> Self {
        let mut abs_base_path = base_path.clone();

        abs_base_path.push(PathBuf::from(self.path.value()));
        let abs_base_path = LitStr::new(pathbuf_to_str(&abs_base_path), self.path.span());

        let sub_paths = self.sub_paths.clone();
        let sub_paths = sub_paths.map(|sp| {
            sp.into_iter()
                .map(|p| {
                    let mut abs_sub_path = base_path.clone();
                    abs_sub_path.push(PathBuf::from(p.value()));
                    LitStr::new(pathbuf_to_str(&abs_sub_path), p.span())
                })
                .collect::<Vec<_>>()
        });

        Self {
            path: abs_base_path,
            sub_paths,
            ..self.clone()
        }
    }

    /// With the the data in `self`, read in the template file and apply any substitutions
    pub fn generate_toml_table(&self) -> Result<toml::Table, pm2::TokenStream> {
        let template_toml = read_litstr_to_toml(&self.path)?;

        let substitute_file = match &self.sub_paths {
            Some(paths) => {
                let mut res_sub = None;

                for sub_lit in paths.iter() {
                    let sub_toml = read_litstr_to_toml(sub_lit)?;
                    // check if use is set to true
                    match sub_toml.contains_key("use") {
                        true => {
                            let (_, use_val) =
                                sub_toml.get_key_value("use").expect("already checked");
                            if let toml::Value::Boolean(true) = use_val {
                                res_sub = Some(sub_toml);
                                break;
                            }
                        }
                        false => (),
                    }
                }

                res_sub
            }
            None => None,
        };

        let merged = match substitute_file {
            Some(sf) => merge_tables(&template_toml, &sf),
            None => template_toml,
        };

        Ok(merged)
    }
}

/// Merge a toml template with a changes table. Changes will set/overwrite values in the template.
fn merge_tables(template: &toml::Table, changes: &toml::Table) -> toml::Table {
    let mut merged_table = template.clone();

    for (key, value) in changes.iter() {
        if let Some(existing_value) = merged_table.get_mut(key) {
            if let Some(existing_table) = existing_value.as_table_mut() {
                if let Some(changes_table) = value.as_table() {
                    // Recursively merge the tables
                    let merged_subtable = merge_tables(existing_table, changes_table);
                    *existing_value = toml::Value::Table(merged_subtable);
                    continue;
                }
            }
        }

        // Update the value directly if it doesn't exist in the template or cannot be merged
        merged_table.insert(key.clone(), value.clone());
    }

    merged_table
}

fn pathbuf_to_str(input: &PathBuf) -> &str {
    input.to_str().expect("failed to convert path to str")
}

/// Read in a litstr path to a toml file, return an error tokenstream if it fails.
fn read_litstr_to_toml(litstr: &LitStr) -> Result<toml::Table, pm2::TokenStream> {
    let path = PathBuf::from(litstr.value());
    let file = match fs::read_to_string(path) {
        Ok(tf) => tf,
        Err(e) => {
            return Err(syn::Error::new(litstr.span(), e.to_string())
                .to_compile_error()
                .to_token_stream());
        }
    };

    let template_toml: toml::Table = match toml::from_str(&file) {
        Ok(tt) => tt,
        Err(e) => {
            return Err(syn::Error::new(litstr.span(), e.to_string())
                .to_compile_error()
                .to_token_stream());
        }
    };

    Ok(template_toml)
}

#[cfg(test)]
mod tests {}
