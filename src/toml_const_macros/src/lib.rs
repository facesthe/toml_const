#![allow(unused)]

mod custom_struct;
mod parse;

use std::{
    any::Any,
    fs,
    path::PathBuf,
    time::{self, UNIX_EPOCH},
};

use custom_struct::{Instantiate, Key, TableTypeDef, find_all_inner_tables, to_camel_case};
use proc_macro::{self as pm, Literal};
use proc_macro2 as pm2;

use parse::MacroInput;
use quote::{ToTokens, quote};
use syn::{Expr, Lit, LitStr, parse_macro_input};

/// Instantiate a const definition of the contents from a TOML file.
///
/// This macro resolves paths relative to the first parent directory containing a `Cargo.toml` file.
#[proc_macro]
pub fn toml_const(input: pm::TokenStream) -> pm::TokenStream {
    let input: MacroInput = parse_macro_input!(input);

    let manifest_path =
        std::env::var("CARGO_MANIFEST_DIR").expect("manifest dir variable must exist");
    let manifest_path = PathBuf::from(manifest_path);
    assert!(manifest_path.is_dir());
    let abs_manifest_path = manifest_path
        .canonicalize()
        .expect("path must canonicalize");

    let collected = input.to_const_defs(&abs_manifest_path);
    let absolute = input.to_abs_path(&abs_manifest_path);

    quote! {
        #collected

        toml_const::toml_const_inner_crate! {
            #absolute
        }
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
    let input: MacroInput = parse_macro_input!(input);

    let ws_dir = std::env::current_dir()
        .expect("current directory must exist")
        .to_string_lossy()
        .to_string();

    let mut ws_path = PathBuf::from(ws_dir);
    assert!(ws_path.is_dir());
    let abs_ws_path = ws_path.canonicalize().expect("path must canonicalize");

    let collected = input.to_const_defs(&abs_ws_path);
    let absolute = input.to_abs_path(&abs_ws_path);

    quote! {
        #collected

        toml_const::toml_const_inner_ws! {
            #absolute
        }
    }
    .into()
}

/// Inner method to be used by [toml_const] macro
#[proc_macro]
pub fn toml_const_inner_crate(input: pm::TokenStream) -> pm::TokenStream {
    let input: MacroInput = parse_macro_input!(input);

    let toml_table = match input.generate_toml_table() {
        Ok(tt) => tt,
        Err(e) => return e.into(),
    };

    let sub_tables = find_all_inner_tables(toml_table.clone());

    let mut table_defs = sub_tables
        .into_iter()
        .map(|(field, sub_table)| sub_table.table_type_def(field.as_str()))
        .collect::<pm2::TokenStream>();

    let root_table_type = &input.item_ident.to_string();
    let root_table_def = toml_table.table_type_def(&root_table_type);

    let instantiation = toml_table.instantiate(Key::Var(&root_table_type));

    let pub_token = if input.is_pub {
        quote! {pub}
    } else {
        quote! {}
    };

    quote! {
        #root_table_def

        #table_defs

        #instantiation
    }
    .into()
}

/// Inner method to be used by [toml_const_ws] macro
#[proc_macro]
pub fn toml_const_inner_ws(input: pm::TokenStream) -> pm::TokenStream {
    let input: MacroInput = parse_macro_input!(input);

    quote! {}.into()
}

/// Test if invocation of include_str! causes the macro to re-run
#[proc_macro]
pub fn test(input: pm::TokenStream) -> pm::TokenStream {
    let input = parse_macro_input!(input as LitStr);

    quote! {
        const _: &'static str = include_str!(#input);
        macros::test_inner!();
    }
    .into()
}

/// Test if invocation of include_str! causes the macro to re-run
#[proc_macro]
pub fn test_inner(input: pm::TokenStream) -> pm::TokenStream {
    // let input = parse_macro_input!(input as LitStr);

    let time_now = time::SystemTime::now();
    let since = time_now
        .duration_since(time::UNIX_EPOCH)
        .expect("must be after epoch");
    let seconds = since.as_secs();

    let current_dir = std::env::current_dir()
        .expect("failed to get current directory")
        .to_string_lossy()
        .to_string();

    quote! {
        const TIME: u64 = #seconds;
        const CUR_DIR: &'static str = #current_dir;
    }
    .into()
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
