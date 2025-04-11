//! Custom struct generation crate.
//!
//! A toml table is converted to a custom struct.
//! The identifier of the struct is used as the struct's type.

use std::borrow::Cow;

use proc_macro2::{self as pm2, Span};
use quote::quote;
use syn::{Ident, punctuated::Punctuated};

/// Chars to replace when converting to an identifier.
const REPLACE_CHARS: &[char] = &[' ', '-', '_', ':', '.', '/', '\\', '"'];

/// Generate the struct definition for arbitrary [toml::Table]s.
///
/// This trait mainly applies to toml tables.
/// Field names remain as SCREAMING_SNAKE_CASE, as they point to static items.
pub trait TableTypeDef {
    fn table_type_def(&self, key: &Key<'_>) -> pm2::TokenStream;
}

/// Return the type of the value.
pub trait ValueType {
    fn value_type(&self, key: &str, parent_ident: &Ident) -> pm2::TokenStream;
}

/// Generate the instantiation of an item. This can be a custom struct or a simple value.
/// If a key is provided, the instantiation will be in a field-value pair.
///
/// Keys are not provided if:
/// - the table is the root table
/// - the value is defined as an element in an array
///
/// This is basically a wrapper around [quote::ToTokens].
pub trait Instantiate {
    fn instantiate(&self, key: Key<'_>, parents: Vec<Ident>) -> pm2::TokenStream;
}

/// Create identifiers for variables and types from a string.
pub trait ConstIdentDef {
    /// Create a valid variable identifier, formatted as SCREAMING_SNAKE_CASE.
    fn to_variable_ident(&self) -> String;

    /// Create a valid module identifier, formatted as snake_case.
    fn to_module_ident(&self) -> String {
        self.to_variable_ident().to_lowercase()
    }

    /// Create a valid type identifier, formatted as PascalCase.
    fn to_type_ident(&self) -> String;

    /// Create an array type identifier formattedas PascalCase.
    fn to_array_type_ident(&self) -> String {
        format!("{}Item", self.to_type_ident())
    }
}

impl<T> ConstIdentDef for T
where
    T: AsRef<str>,
{
    fn to_variable_ident(&self) -> String {
        let self_ref = self.as_ref();

        let inter = self_ref.replace(REPLACE_CHARS, "_");

        let inter = inter
            .split('_')
            .map(|item| item.to_uppercase())
            .collect::<Vec<_>>()
            .join("_");

        match inter.starts_with(char::is_numeric) {
            true => format!("_{}", inter),
            false => inter,
        }
    }

    fn to_type_ident(&self) -> String {
        let inter = self.as_ref().replace(REPLACE_CHARS, "_");

        let inter = match inter.contains("_") {
            true => inter
                .split('_')
                .map(|item| {
                    let mut chars = item.chars();

                    match chars.next() {
                        Some(c) => {
                            let first_char = c.to_ascii_uppercase();
                            let rest = chars.collect::<String>().to_ascii_lowercase();
                            format!("{}{}", first_char, rest)
                        }
                        None => String::new(),
                    }
                })
                .collect::<String>(),
            false => {
                // split at a capital letter, but preserve the letter
                let inter = inter.chars().fold(String::new(), |mut acc, c| {
                    if c.is_uppercase() && !acc.is_empty() {
                        acc.push('_');
                    }
                    acc.push(c);
                    acc
                });

                inter
                    .split("_")
                    .map(|item| {
                        let mut chars = item.chars();

                        match chars.next() {
                            Some(c) => {
                                let first_char = c.to_ascii_uppercase();
                                let rest = chars.collect::<String>().to_ascii_lowercase();
                                format!("{}{}", first_char, rest)
                            }
                            None => String::new(),
                        }
                    })
                    .collect::<String>()

                // todo!()
            }
        };

        match inter.starts_with(char::is_numeric) {
            true => format!("_{}", inter),
            false => inter,
        }
    }
}

/// A key that accompanies an instantiation
#[derive(Clone, Copy)]
pub enum Key<'a> {
    /// The item is instantiated as an element in an array.
    ///
    /// For these cases, the element type identifier depends on the parent key.
    Element(&'a str),

    /// The key is a struct field name.
    Field(&'a str),

    /// The key is a variable identifier.
    /// This only applies to tables.
    Var(&'a Ident),
}

impl ValueType for toml::Value {
    fn value_type(&self, key: &str, parent_ident: &Ident) -> proc_macro2::TokenStream {
        match &self {
            toml::Value::String(_) => quote! { &'static str },
            toml::Value::Integer(_) => quote! { i64 },
            toml::Value::Float(_) => quote! { f64 },
            toml::Value::Boolean(_) => quote! { bool },
            toml::Value::Datetime(_) => quote! { toml_const::Datetime },
            // array types have "Item" as a suffix
            toml::Value::Array(values) => {
                let value_type = match values.len() {
                    0 => quote! { toml_const::Array<toml_const::Empty> },
                    _ => {
                        let first = &values[0];
                        first.value_type(&key.to_array_type_ident(), parent_ident)
                    }
                };

                quote! { toml_const::Array<#value_type> }
            }
            toml::Value::Table(_) => {
                let type_name = key.to_type_ident();
                let type_name = pm2::Ident::new(&type_name, proc_macro2::Span::call_site());

                quote! { #parent_ident :: #type_name }
            }
        }
    }
}

impl Instantiate for toml::Value {
    fn instantiate(&self, key: Key, parents: Vec<Ident>) -> proc_macro2::TokenStream {
        use toml::Value::*;

        // for predefined types
        let inner = key.value();
        let field_name = inner.to_variable_ident();

        let field = pm2::Ident::new(&field_name, proc_macro2::Span::call_site());

        match (self, key) {
            (
                String(_) | Integer(_) | Float(_) | Boolean(_) | Datetime(_) | Array(_),
                Key::Var(_),
            ) => unimplemented!("only tables can be a root item and assigned to a variable"),

            // cases when items are instantiated as fields in an array
            (toml::Value::String(val), Key::Element(_)) => quote! { #val },
            (toml::Value::Integer(val), Key::Element(_)) => quote! { #val },
            (toml::Value::Float(val), Key::Element(_)) => quote! { #val },
            (toml::Value::Boolean(val), Key::Element(_)) => quote! { #val },

            // field-value pair instantiation
            (toml::Value::String(val), Key::Field(_)) => quote! { #field: #val },
            (toml::Value::Integer(val), Key::Field(_)) => quote! { #field: #val },
            (toml::Value::Float(val), Key::Field(_)) => quote! { #field: #val },
            (toml::Value::Boolean(val), Key::Field(_)) => quote! { #field: #val },

            // items with inner impls
            (toml::Value::Datetime(datetime), k) => datetime.instantiate(k, vec![]),
            (toml::Value::Array(values), k) => values.instantiate(k, parents),
            (toml::Value::Table(map), k) => map.instantiate(k, parents),
        }
    }
}

impl Instantiate for toml::Table {
    fn instantiate(&self, key: Key, parents: Vec<Ident>) -> proc_macro2::TokenStream {
        // let inner = key.value();
        let field_name = key.value();

        let field = pm2::Ident::new(
            &field_name.to_variable_ident(),
            proc_macro2::Span::call_site(),
        );
        let table_type = syn::Ident::new(&field_name.to_type_ident(), Span::call_site());
        let table_mod = syn::Ident::new(&field_name.to_module_ident(), Span::call_site());

        let mut parents_inner = parents.clone();
        parents_inner.push(table_mod);

        let fields = self
            .iter()
            .map(|(f_key, f_val)| f_val.instantiate(Key::Field(f_key), parents_inner.clone()))
            .collect::<Punctuated<pm2::TokenStream, syn::Token![,]>>();

        let parent_mod_path = match parents.len() {
            0 => quote! {},
            _ => {
                let mod_path = parents
                    .into_iter()
                    .collect::<Punctuated<Ident, syn::Token![::]>>();

                quote! {
                    #mod_path ::
                }
            }
        };

        match key {
            Key::Element(_) => {
                let table_type =
                    pm2::Ident::new(&field_name.to_array_type_ident(), Span::call_site());

                quote! { #parent_mod_path #table_type { #fields } }
            }
            Key::Field(_) => quote! {
                #field: #parent_mod_path #table_type {
                    #fields
                }
            },

            Key::Var(v) => {
                let var_name = v.to_string();
                let new_var = Ident::new(&var_name.to_variable_ident(), v.span());
                quote! {
                    #new_var: #table_type = #table_type {
                        #fields
                    };
                }
            }
        }
    }
}

impl Instantiate for toml::value::Array {
    fn instantiate(&self, key: Key<'_>, parents: Vec<Ident>) -> proc_macro2::TokenStream {
        let inner = key.value();
        let elem_key = Key::Element(&inner);

        let elements = self
            .iter()
            .map(|item| item.instantiate(elem_key, parents.clone()))
            .collect::<Punctuated<pm2::TokenStream, syn::Token![,]>>();

        match key {
            Key::Element(_) => quote! {toml_const::Array(&[#elements])},
            Key::Field(k) => {
                let field_key = k.to_variable_ident();
                let field = pm2::Ident::new(&field_key, proc_macro2::Span::call_site());

                quote! { #field: toml_const::Array(&[#elements])}
            }
            Key::Var(_) => unimplemented!("arrays cannot be instantiated as variables"),
        }

        // todo!("array impl in progress")
    }
}

// datetime structs do not require a key, as they are already defined.
impl Instantiate for toml::value::Datetime {
    fn instantiate(&self, k: Key, parents: Vec<Ident>) -> proc_macro2::TokenStream {
        // this is technically not required
        let inner = k.value();
        let key_inner = Key::Element(&inner);

        let date_value = match self.date {
            Some(date) => {
                let instantiated = date.instantiate(key_inner, parents.clone());
                quote! { Option::Some(#instantiated) }
            }
            None => quote! {Option::None},
        };

        let time_value = match self.time {
            Some(time) => {
                let instantiated = time.instantiate(key_inner, parents.clone());
                quote! { Option::Some(#instantiated) }
            }
            None => quote! {Option::None},
        };

        let offset_value = match self.offset {
            Some(offset) => {
                let instantiated = offset.instantiate(key_inner, parents.clone());
                quote! { Option::Some(#instantiated) }
            }
            None => quote! {Option::None},
        };

        let self_value = quote! {
            toml_const::Datetime {
                date: #date_value,
                time: #time_value,
                offset: #offset_value
            }
        };

        match k {
            Key::Element(_) => self_value,
            Key::Field(val) => {
                let field_key = val.to_variable_ident();
                let field = pm2::Ident::new(&field_key, proc_macro2::Span::call_site());

                quote! { #field: #self_value }
            }
            Key::Var(_) => unimplemented!("datetime cannot be instantiated as a variable"),
        }
    }
}

// sub structs do not require key, they implement `Key::Element`.
impl Instantiate for toml::value::Date {
    fn instantiate(&self, _: Key<'_>, _: Vec<Ident>) -> proc_macro2::TokenStream {
        let year = self.year;
        let month = self.month;
        let day = self.day;

        quote! {
            toml_const::Date {
                year: #year,
                month: #month,
                day: #day
            }
        }
    }
}

impl Instantiate for toml::value::Time {
    fn instantiate(&self, _: Key<'_>, _: Vec<Ident>) -> proc_macro2::TokenStream {
        let hour = self.hour;
        let minute = self.minute;
        let second = self.second;
        let nanosecond = self.nanosecond;

        quote! {
            toml_const::Time {
                hour: #hour,
                minute: #minute,
                second: #second,
                nanosecond: #nanosecond
            }
        }
    }
}

impl Instantiate for toml::value::Offset {
    fn instantiate(&self, _: Key<'_>, _: Vec<Ident>) -> proc_macro2::TokenStream {
        match self {
            toml::value::Offset::Z => quote! { toml_const::Offset::Z },
            toml::value::Offset::Custom { minutes } => quote! {
                toml_const::Offset::Custom {
                    minutes: #minutes
                }
            },
        }
    }
}

impl<'a> Key<'a> {
    /// Return the value contained in the key.
    pub fn value(&'a self) -> Cow<'a, str> {
        match self {
            Key::Element(v) => Cow::Borrowed(v),
            Key::Field(v) => Cow::Borrowed(v),
            Key::Var(v) => Cow::Owned(v.to_string()),
        }
    }
}

impl TableTypeDef for toml::Table {
    fn table_type_def(&self, key: &Key<'_>) -> proc_macro2::TokenStream {
        let mod_self = key.value().to_module_ident();
        let mod_self = pm2::Ident::new(&mod_self, proc_macro2::Span::call_site());

        let fields = self
            .iter()
            .map(|(key, val)| {
                let field_name = key.to_variable_ident();

                let field_type = val.value_type(key, &mod_self);
                let field_name = pm2::Ident::new(&field_name, proc_macro2::Span::call_site());

                quote! { pub #field_name: #field_type }
            })
            .collect::<Punctuated<pm2::TokenStream, syn::Token![,]>>();

        let table_type = match key {
            Key::Element(e) => e.to_array_type_ident(),
            Key::Field(f) => f.to_type_ident(),
            Key::Var(ident) => ident.to_string().to_type_ident(),
        };

        let table_type = pm2::Ident::new(&table_type, proc_macro2::Span::call_site());

        quote! {
            #[allow(non_snake_case, unused)]
            #[derive(Clone, Copy, Debug)]
            pub struct #table_type {
                #fields
            }
        }
    }
}

/// Descend down the current toml table and define self and all inner tables as structs.
///
/// Inner tables are defined in a module named after their parent table.
/// This is done so identically named sub-tables can co-exist in the same file.
pub fn def_inner_tables(table: &toml::Table, key: &Key<'_>) -> pm2::TokenStream {
    let self_def = table.table_type_def(key);

    let inner_defs = table
        .iter()
        .filter_map(|(key, val)| match val {
            toml::Value::Array(arr) => match arr.len() {
                0 => Option::<pm2::TokenStream>::None,
                _ => {
                    let first = &arr[0];

                    if let toml::Value::Table(t) = first {
                        Some(def_inner_tables(t, &Key::Element(key)))
                    } else {
                        None
                    }
                }
            },
            toml::Value::Table(tab) => {
                let inner = def_inner_tables(tab, &Key::Field(key));

                Some(inner)
            }
            _ => None,
        })
        .collect::<pm2::TokenStream>();

    let mod_self = key.value().to_module_ident();
    let mod_self = pm2::Ident::new(&mod_self, proc_macro2::Span::call_site());

    quote! {
        #self_def

        pub mod #mod_self {
            use super::toml_const;

            #inner_defs
        }
    }
}

/// Inner method for [def_inner_tables].
fn _def_inner_tables() -> pm2::TokenStream {
    todo!()
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::*;

    // #[test]
    // fn test_screaming_snake_case() {
    //     let input = "hello world";
    //     let res = input.to_variable_ident();
    //     assert_eq!(res, "HELLO_WORLD");
    // }

    // #[test]
    // fn test_camel_case() {
    //     let input = "hello world";
    //     let res = to_camel_case(input);
    //     assert_eq!(res, "HelloWorld");
    // }

    #[test]
    fn test_sub_tables() {
        let cargo_manifest = include_str!("../../../Cargo.toml");
        let toml: toml::Table = toml::Table::from_str(cargo_manifest).unwrap();

        let table_defs = def_inner_tables(
            &toml,
            &Key::Var(&Ident::new("ROOT_TABLE", Span::call_site())),
        );

        println!("Table definitions: {}", table_defs);

        // let tables = find_all_inner_tables(toml.clone());

        // for t in tables {
        //     println!("{:#?}", t);
        // }

        // let table_def = toml.table_type_def("ROOT_TABLE");

        // println!("Table definition: {}", table_def.to_string());

        // let table_inst = toml.instantiate(Key::Var(&Ident::new("ROOT_TABLE", Span::call_site())));
        // println!("Table instantiation: {}", table_inst.to_string());
    }

    #[test]
    fn test_instantiation() {
        let cargo_manifest = include_str!("../../../Cargo.toml");
        let toml: toml::Table = toml::Table::from_str(cargo_manifest).unwrap();

        let root_ident = Ident::new("ROOT_TABLE", Span::call_site());
        let instantiation = toml.instantiate(Key::Var(&root_ident), vec![]);

        println!("Table instantiation: {}", instantiation);
    }

    #[test]
    fn test_split_pascal_case() {
        let inter = "PascalCase";

        let inter = inter.chars().fold(String::new(), |mut acc, c| {
            if c.is_uppercase() && !acc.is_empty() {
                acc.push('_');
            }
            acc.push(c);
            acc
        });

        println!("inter: {inter}");

        let inter = "Pascal";

        let inter = inter.chars().fold(String::new(), |mut acc, c| {
            if c.is_uppercase() && !acc.is_empty() {
                acc.push('_');
            }
            acc.push(c);
            acc
        });

        println!("inter: {inter}");
    }
}
