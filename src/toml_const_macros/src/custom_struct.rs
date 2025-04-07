//! Custom struct generation crate.
//!
//! A toml table is converted to a custom struct.
//! The identifier of the struct is used as the struct's type.

use proc_macro2::{self as pm2, Span};
use quote::{ToTokens, quote};
use syn::{LitStr, punctuated::Punctuated};

/// Chars to replace when converting to screaming snake case.
const REPLACE_CHARS: &[char] = &[' ', '-', '_', ':', '.', '/', '\\', '"'];

/// Generate the struct definition for arbitrary [toml::Table]s.
///
/// This trait mainly applies to toml tables.
/// Field names remain as SCREAMING_SNAKE_CASE, as they point to static items.
pub trait TableTypeDef {
    fn table_type_def(&self, key: &str) -> pm2::TokenStream;
}

/// Return the type of the value.
pub trait ValueType {
    fn value_type(&self, key: &str) -> pm2::TokenStream;
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
    fn instantiate(&self, key: Key<'_>) -> pm2::TokenStream;
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
    Var(&'a str),
}

impl TableTypeDef for toml::Table {
    fn table_type_def(&self, key: &str) -> proc_macro2::TokenStream {
        let fields = self
            .iter()
            .map(|(key, val)| {
                let field_name = to_screaming_snake_case(key);

                let field_type = val.value_type(key);
                let field_name = pm2::Ident::new(&field_name, proc_macro2::Span::call_site());

                quote! { pub #field_name: #field_type }
            })
            .collect::<Punctuated<pm2::TokenStream, syn::Token![,]>>();

        let table_type = to_camel_case(key);
        let table_type = pm2::Ident::new(&table_type, proc_macro2::Span::call_site());

        quote! {
            #[allow(non_snake_case, unused)]
            pub struct #table_type {
                #fields
            }
        }
    }
}

impl ValueType for toml::Value {
    fn value_type(&self, key: &str) -> proc_macro2::TokenStream {
        match &self {
            toml::Value::String(_) => quote! { &'static str },
            toml::Value::Integer(_) => quote! { i64 },
            toml::Value::Float(_) => quote! { f64 },
            toml::Value::Boolean(_) => quote! { bool },
            toml::Value::Datetime(datetime) => quote! { toml_const::Datetime },
            // array types have "Item" as a suffix
            toml::Value::Array(values) => {
                let value_type = match values.len() {
                    0 => quote! { toml_const::Array<toml_const::Empty> },
                    rest => {
                        let first = &values[0];
                        let value_key = format!("{}_item", key);
                        first.value_type(&value_key)
                    }
                };

                quote! { toml_const::Array<#value_type> }
            }
            toml::Value::Table(map) => {
                let type_name = to_camel_case(key);
                let type_name = pm2::Ident::new(&type_name, proc_macro2::Span::call_site());

                quote! { #type_name }
            }
        }
    }
}

impl Instantiate for toml::Value {
    fn instantiate(&self, key: Key) -> proc_macro2::TokenStream {
        use toml::Value::*;

        // for predefined types
        let field_name = to_screaming_snake_case(key.value());
        let type_name = to_camel_case(key.value());

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
            (toml::Value::Datetime(datetime), k) => datetime.instantiate(k),
            (toml::Value::Array(values), k) => values.instantiate(k),
            (toml::Value::Table(map), k) => map.instantiate(k),
        }
    }
}

impl Instantiate for toml::Table {
    fn instantiate(&self, key: Key) -> proc_macro2::TokenStream {
        let field_name = to_screaming_snake_case(key.value());

        let field = pm2::Ident::new(&field_name, proc_macro2::Span::call_site());

        let table_type = syn::Ident::new(&to_camel_case(&field_name), Span::call_site());

        let fields = self
            .iter()
            .map(|(f_key, f_val)| f_val.instantiate(Key::Field(f_key)))
            .collect::<Punctuated<pm2::TokenStream, syn::Token![,]>>();

        match key {
            Key::Element(_) => {
                let table_type = pm2::Ident::new(
                    &to_camel_case(&format!("{}_item", field_name)),
                    Span::call_site(),
                );

                quote! { #table_type { #fields } }
            }
            Key::Field(_) => quote! {
                #field: #table_type {
                    #fields
                }
            },
            Key::Var(_) => quote! {
                pub static #field: #table_type = #table_type {
                    #fields
                };
            },
        }
    }
}

impl Instantiate for toml::value::Array {
    fn instantiate(&self, key: Key<'_>) -> proc_macro2::TokenStream {
        // asd

        let elem_key = Key::Element(key.value());

        let elements = self
            .iter()
            .map(|item| item.instantiate(elem_key))
            .collect::<Punctuated<pm2::TokenStream, syn::Token![,]>>();

        match key {
            Key::Element(_) => quote! {toml_const::Array(&[#elements])},
            Key::Field(k) => {
                let field_key = to_screaming_snake_case(k);
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
    fn instantiate(&self, k: Key) -> proc_macro2::TokenStream {
        // this is technically not required
        let key_inner = Key::Element(k.value());

        let date_value = match self.date {
            Some(date) => {
                let instantiated = date.instantiate(key_inner);
                quote! { Option::Some(#instantiated) }
            }
            None => quote! {Option::None},
        };

        let time_value = match self.time {
            Some(time) => {
                let instantiated = time.instantiate(key_inner);
                quote! { Option::Some(#instantiated) }
            }
            None => quote! {Option::None},
        };

        let offset_value = match self.offset {
            Some(offset) => {
                let instantiated = offset.instantiate(key_inner);
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
                let field_key = to_screaming_snake_case(val);
                let field = pm2::Ident::new(&field_key, proc_macro2::Span::call_site());

                quote! { #field: #self_value }
            }
            Key::Var(_) => unimplemented!("datetime cannot be instantiated as a variable"),
        }
    }
}

// sub structs do not require key, they implement `Key::Element`.
impl Instantiate for toml::value::Date {
    fn instantiate(&self, _: Key<'_>) -> proc_macro2::TokenStream {
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
    fn instantiate(&self, _: Key<'_>) -> proc_macro2::TokenStream {
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
    fn instantiate(&self, _: Key<'_>) -> proc_macro2::TokenStream {
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
    pub fn value(&'a self) -> &'a str {
        match self {
            Key::Element(v) => v,
            Key::Field(v) => v,
            Key::Var(v) => v,
        }
    }
}

/// Convert an item to screaming snake case.
///
/// Replaces all invalid characters with `_`.
fn to_screaming_snake_case(name: &str) -> String {
    let inter = name.replace(REPLACE_CHARS, "_");

    inter
        .split('_')
        .map(|item| item.to_uppercase())
        .collect::<Vec<_>>()
        .join("_")
}

/// Create a type identifier from a name
pub fn to_camel_case(name: &str) -> String {
    let inter = name.replace(REPLACE_CHARS, "_");

    inter
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
        .collect::<String>()
}

/// Return all inner tables except the root table.
pub fn find_all_inner_tables(table: toml::Table) -> Vec<(String, toml::Table)> {
    let mut tables = vec![];

    for (table_key, value) in table.into_iter() {
        match value {
            toml::Value::Table(sub_table) => {
                let sub_tables = find_all_tables(table_key, sub_table);

                tables.extend(sub_tables);
            }
            toml::Value::Array(arr) => {
                // we infer the array type from the first element
                if let Some(toml::Value::Table(t)) = arr.first() {
                    // pass in the array key
                    let arr_key = format!("{}_item", table_key);
                    let sub_tables = find_all_tables(arr_key, t.clone());

                    tables.extend(sub_tables);
                }
            }
            _ => (),
        }
    }

    tables
}

/// Inner function for [find_all_inner_tables].
fn find_all_tables(key: String, input: toml::Table) -> Vec<(String, toml::Table)> {
    let mut tables = vec![];

    for (table_key, value) in input.iter() {
        match value {
            toml::Value::Table(sub_table) => {
                let sub_tables = find_all_tables(table_key.to_owned(), sub_table.clone());

                tables.extend(sub_tables);
            }
            toml::Value::Array(arr) => {
                // we infer the array type from the first element
                if let Some(toml::Value::Table(t)) = arr.first() {
                    // pass in the array key
                    let arr_key = format!("{}_item", table_key);
                    let sub_tables = find_all_tables(arr_key, t.clone());

                    tables.extend(sub_tables);
                }
            }
            _ => (),
        }
    }

    tables.push((key, input));

    tables
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::*;

    #[test]
    fn test_screaming_snake_case() {
        let input = "hello world";
        let res = to_screaming_snake_case(input);
        assert_eq!(res, "HELLO_WORLD");
    }

    #[test]
    fn test_camel_case() {
        let input = "hello world";
        let res = to_camel_case(input);
        assert_eq!(res, "HelloWorld");
    }

    #[test]
    fn test_sub_tables() {
        let cargo_manifest = include_str!("../../../Cargo.toml");
        let toml: toml::Table = toml::Table::from_str(cargo_manifest).unwrap();

        let tables = find_all_inner_tables(toml.clone());

        for t in tables {
            println!("{:#?}", t);
        }

        let table_def = toml.table_type_def("ROOT_TABLE");

        println!("Table definition: {}", table_def.to_string());

        let table_inst = toml.instantiate(Key::Var("ROOT_TABLE"));
        println!("Table instantiation: {}", table_inst.to_string());
    }
}
