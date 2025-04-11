mod check;
mod custom_struct;
mod parse;

use std::path::PathBuf;

use custom_struct::{Instantiate, Key, def_inner_tables};
use proc_macro as pm;
use proc_macro2::{self as pm2, Span};

use parse::{MacroInput, MultipleMacroInput};
use quote::quote;
use syn::parse_macro_input;

/// Instantiate a const definition of the contents from a TOML file.
///
/// This macro resolves paths relative to the first parent directory containing a `Cargo.toml` file.
#[proc_macro]
pub fn toml_const(input: pm::TokenStream) -> pm::TokenStream {
    let input: MultipleMacroInput = parse_macro_input!(input);

    let manifest_path =
        std::env::var("CARGO_MANIFEST_DIR").expect("manifest dir variable must exist");
    let manifest_path = PathBuf::from(manifest_path);
    assert!(manifest_path.is_dir());
    let abs_manifest_path = manifest_path
        .canonicalize()
        .expect("path must canonicalize");

    let const_defs = input
        .0
        .iter()
        .map(|i| i.to_const_defs(&abs_manifest_path))
        .collect::<pm2::TokenStream>();

    let inner_calls = input
        .0
        .iter()
        .map(|i| {
            let absolute = i.to_abs_path(&abs_manifest_path);
            quote! {
                toml_const::toml_const_inner! {
                    #absolute
                }
            }
        })
        .collect::<pm2::TokenStream>();

    quote! {
        #const_defs

        #inner_calls
    }
    .into()
}

/// Instantiate a const definition of the contents from a TOML file.
///
/// If this macro is used in a workspace, it will resolve paths relative to the workspace's `Cargo.toml`.
///
/// If this macro is used in a crate, it will resolve paths relative to the crate's `Cargo.toml`.
#[proc_macro]
pub fn toml_const_ws(input: pm::TokenStream) -> pm::TokenStream {
    let input: MultipleMacroInput = parse_macro_input!(input);

    let ws_dir = std::env::current_dir()
        .expect("current directory must exist")
        .to_string_lossy()
        .to_string();

    let ws_path = PathBuf::from(ws_dir);
    assert!(ws_path.is_dir());
    let abs_ws_path = ws_path.canonicalize().expect("path must canonicalize");

    let const_defs = input
        .0
        .iter()
        .map(|i| i.to_const_defs(&abs_ws_path))
        .collect::<pm2::TokenStream>();

    // let collected = input.to_const_defs(&abs_ws_path);
    let inner_calls = input
        .0
        .iter()
        .map(|i| {
            let absolute = i.to_abs_path(&abs_ws_path);
            quote! {
                toml_const::toml_const_inner! {
                    #absolute
                }
            }
        })
        .collect::<pm2::TokenStream>();

    quote! {
        #const_defs

        #inner_calls
    }
    .into()
}

/// Inner method call generated by public macros
#[doc(hidden)]
#[proc_macro]
pub fn toml_const_inner(input: pm::TokenStream) -> pm::TokenStream {
    let input: MacroInput = parse_macro_input!(input);

    let toml_table = match input.generate_toml_table() {
        Ok(tt) => tt,
        Err(e) => return e.into(),
    };

    match check::check_unauthorized_keys(&toml_table) {
        Ok(_) => (),
        Err(e) => return e.into(),
    }

    match check::check(&toml_table) {
        Ok(_) => (),
        Err(e) => {
            return syn::Error::new(Span::call_site(), e.to_string())
                .to_compile_error()
                .into();
        }
    }

    let table_definitions = def_inner_tables(&toml_table, &Key::Var(&input.item_ident));
    let instantiation = toml_table.instantiate(Key::Var(&input.item_ident), vec![]);

    let pub_token = if input.is_pub {
        quote! {pub}
    } else {
        quote! {}
    };

    quote! {
        #table_definitions

        #pub_token static #instantiation
    }
    .into()
}
