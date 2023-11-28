//! Build.rs main code generation

use core::panic;
use std::{
    collections::HashMap,
    fs::{self, OpenOptions},
    io::Write,
    path::Path,
    process::exit,
    str::FromStr,
};
use toml::{self, Value};

use crate::codegen;

use super::consts::*;

enum Setting {
    Template = 0,
    Debug = 1,
    Deploy = 2,
}

/// This is the main codegen function. Run this inside your `build.rs`!
///
/// ```rust no_run
/// use toml_const::run;
///
/// // main function of your build.rs
/// fn main() {
///     run();
///     // ... rest of your build script
/// }
/// ```
pub fn run() {
    // read in environment variables
    let config_dir = std::env::var(CONFIG_PATH_ENV);
    let template_path = std::env::var(TEMPLATE_ENV);
    let debug_path = std::env::var(DEBUG_ENV);
    let deploy_path = std::env::var(DEPLOY_ENV);
    let generated_path = std::env::var(GENERATED_FILE_PATH_ENV);

    let (config_dir, template_path, debug_path, deploy_path, generated_path) = match (
        config_dir,
        template_path,
        debug_path,
        deploy_path,
        generated_path,
    ) {
        (Ok(path), Ok(temp), Ok(deb), Ok(dep), Ok(gen)) => (path, temp, deb, dep, gen),
        _ => exit(-1),
    };

    let settings_arr = vec![
        format!("{}/{}", config_dir, template_path),
        format!("{}/{}", config_dir, debug_path),
        format!("{}/{}", config_dir, deploy_path),
    ];

    // rerun this file if these files change
    println!("cargo:rerun-if-changed=build.rs");
    // println!("cargo:rerun-if-changed={}", GENERATED_FILE_PATH);
    for s in &settings_arr {
        println!("cargo:rerun-if-changed={}", s);
    }

    let mut settings_contents = Vec::new();

    let template_result = read_append_to_vec(&mut settings_contents, &settings_arr[0]);
    if !template_result {
        panic!("file should exist: {}", settings_arr[0]);
    }

    let deploy_file: bool;
    let debug_file: bool;

    debug_file = read_append_to_vec(&mut settings_contents, &settings_arr[1]);
    deploy_file = read_append_to_vec(&mut settings_contents, &settings_arr[2]);

    let file_to_use: usize; // indexes into settings_arr
    let mut perform_perge: bool = true; // set to false if no debug/deploy config found
    match (debug_file, deploy_file) {
        (true, true) => {
            let debug = toml::Table::from_str(&settings_contents[Setting::Debug as usize]).unwrap();
            let deploy =
                toml::Table::from_str(&settings_contents[Setting::Deploy as usize]).unwrap();

            let debug_use = debug
                .get("use")
                .and_then(|val| match val {
                    Value::Boolean(_b) => Some(_b.to_owned()),
                    _ => None,
                })
                .unwrap_or(false);

            let deploy_use = deploy
                .get("use")
                .and_then(|val| match val {
                    Value::Boolean(_b) => Some(_b.to_owned()),
                    _ => None,
                })
                .unwrap_or(false);

            match (debug_use, deploy_use) {
                (true, true) => file_to_use = Setting::Deploy as usize,
                (true, false) => file_to_use = Setting::Debug as usize,
                (false, true) => file_to_use = Setting::Deploy as usize,
                (false, false) => {
                    println!("cargo:warning=debug/deploy not found or \"use = true\" pair not set. Set this key-value pair inside one configuration file.");
                    perform_perge = false;
                    file_to_use = usize::MAX;
                }
            }
        }
        (true, false) => file_to_use = Setting::Debug as usize,
        (false, true) => file_to_use = Setting::Deploy as usize,
        (false, false) => {
            file_to_use = Setting::Template as usize; // merge into self, effectively doing nothing

            println!("cargo:warning=debug/deploy file missing. At least one file required:");
            println!("cargo:warning=- {}", settings_arr[Setting::Debug as usize]);
            println!("cargo:warning=- {}", settings_arr[Setting::Deploy as usize]);
            println!("cargo:warning=Default settings may cause panics on runtime.");
        }
    }

    let merged = match perform_perge {
        false => toml::Table::from_str(&settings_contents[Setting::Template as usize]).unwrap(),
        true => merge_tables(
            &toml::Table::from_str(&settings_contents[Setting::Template as usize]).unwrap(),
            &toml::Table::from_str(&settings_contents[file_to_use]).unwrap(),
        ),
    };

    // codegen
    let mut _wrapper = codegen::CodeGenWrapper::new(generated_path.clone());

    let hash_table = table_to_flat_hashmap(&merged, None);
    // generate everything except tables (cause they have been flattened)
    let absolute_gen = codegen::generate_absolute_variables(hash_table);
    // generate last level tables (from unflattened OG table)
    let hashmap_gen = codegen::generate_last_level_hashmap(&merged, None);
    let mut gen_file = OpenOptions::new()
        .append(true)
        .open(generated_path)
        .unwrap();

    gen_file.write_all(absolute_gen.as_bytes()).unwrap();

    _wrapper.lazy_static(&mut gen_file);
    gen_file.write_all(hashmap_gen.as_bytes()).unwrap();
}

/// ChatGPT generated
/// Compares base against other.
/// Merges any matching keys from other to base.
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

/// Checks if file exists, and appends to vec.
/// Returns true and appends to vec if file exists,
/// returns false and appends an empty string if file does not exist.
fn read_append_to_vec(vec: &mut Vec<String>, file_path: &str) -> bool {
    if Path::new(file_path).exists() {
        vec.push(fs::read_to_string(file_path).unwrap());
        true
    } else {
        vec.push(format!(""));
        false
    }
}

/// Convert a toml table to a hashmap by flattening
/// All tables are destructured.
///
/// Hashmap values can by of any toml type except table.
fn table_to_flat_hashmap(table: &toml::Table, prefix: Option<&str>) -> HashMap<String, Value> {
    let mut map = HashMap::<String, Value>::new();

    for (key, val) in table.iter() {
        let mut _key = key.to_owned().to_uppercase().replace("-", "_");
        if let Some(pre) = prefix {
            _key = format!("{}_{}", pre, _key);
        }

        if let Value::Table(t) = val {
            let sub_map = table_to_flat_hashmap(t, Some(_key.as_str()));
            map.extend(sub_map);
        } else {
            map.insert(_key, val.to_owned());
        }
    }

    map
}
