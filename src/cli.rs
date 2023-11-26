//! CLI related code, used for generating boilerplate.

use crate::{
    consts::{
        CONFIG_PATH_ENV, CONFIG_TOML_BOILERPLATE, DEBUG_ENV, DEPLOY_ENV, GENERATED_FILE_PATH_ENV,
        TEMPLATE_ENV,
    },
    package_navi::find_cargo_parent,
};
use clap::Parser;

use std::{
    fs::{self, OpenOptions},
    io::{Read, Write},
    path::PathBuf,
    process::ExitCode,
    str::FromStr,
};

use toml::Value;

#[derive(Clone, Debug, Parser)]
pub struct CliArgs {
    #[clap(subcommand)]
    pub command: MainSubCommands,
}

#[derive(Clone, Debug, Parser)]
pub enum MainSubCommands {
    /// Initialize a new project with boilerplate
    Init(Init),
}

#[derive(Clone, Debug, Parser)]
pub struct Init {
    /// Path to Cargo.toml
    #[clap(value_parser)]
    pub manifest_path: String,

    /// Configuration dir for toml files, relative to the root cargo manifest.
    ///
    /// The root manifest can differ from the indicated manifest if it belongs in
    /// a workspace, or is nested as a sub package.
    #[clap(short, long, default_value = ".config/")]
    pub config_path: String,

    /// Path to generated file, relative to the provided manifest path
    #[clap(short, long, default_value = "generated.rs")]
    pub generated_file_path: String,
}

/// Run the CLI
pub fn run() -> ExitCode {
    let args = CliArgs::parse();

    // we only have one subcommand right now
    #[allow(irrefutable_let_patterns)]
    let args = if let MainSubCommands::Init(i) = args.command {
        i
    } else {
        return ExitCode::SUCCESS;
    };

    let cargo_manifest = match fs::read_to_string(&args.manifest_path) {
        Ok(f) => f,
        Err(e) => {
            log::error!("Failed to read cargo manifest: {}", e);
            return ExitCode::FAILURE;
        }
    };

    let table: toml::Table = match toml::from_str(&cargo_manifest) {
        Ok(t) => t,
        Err(e) => {
            log::error!("Failed to parse manifest into toml: {}", e);
            return ExitCode::FAILURE;
        }
    };

    // get the package name
    let t = match table.get("package").and_then(|t| t.get("name")) {
        Some(t) => t,
        None => {
            log::error!("Cargo manifest does not have a package name. The manifest specified may be a workspace.");

            return ExitCode::FAILURE;
        }
    };

    let package_name = match t {
        Value::String(p) => p,
        _ => {
            log::error!("Cargo package name needs to be a string");
            return ExitCode::FAILURE;
        }
    };

    let template_name = format!("{}.template.toml", package_name);
    let debug_name = format!("{}.debug.toml", package_name);
    let deploy_name = format!("{}.deploy.toml", package_name);

    // write env variables into cargo config
    let (cargo_project_root, cargo_dot_config_file, toml_config_dir, generated_file) = {
        let mut cargo_project_directory = PathBuf::from_str(&args.manifest_path)
            .unwrap()
            .canonicalize()
            .unwrap()
            .parent()
            .expect("failed to get cargo manifest directory")
            .to_owned();

        let mut generated_file = cargo_project_directory.clone();
        generated_file.push(args.generated_file_path);
        generated_file = generated_file
            .strip_prefix(&cargo_project_directory)
            .unwrap()
            .to_path_buf();

        let mut toml_config_dir = cargo_project_directory.clone();
        toml_config_dir.push(args.config_path);
        toml_config_dir = toml_config_dir
            .strip_prefix(&cargo_project_directory)
            .unwrap()
            .to_path_buf();

        // the .cargo/config.toml lives in the project root (top level dir that contains a Cargo.toml file)
        let mut cargo_config_dir = match find_cargo_parent(&cargo_project_directory) {
            Some(root) => {
                let root_parent = root.parent().unwrap().to_path_buf();
                cargo_project_directory = root_parent.clone();
                root_parent
            }
            None => cargo_project_directory.clone(),
        };

        cargo_config_dir.push(".cargo");

        println!("{:?}", cargo_config_dir);

        fs::create_dir_all(&cargo_config_dir).unwrap();

        cargo_config_dir.push("config.toml");
        (
            cargo_project_directory,
            cargo_config_dir,
            toml_config_dir,
            generated_file,
        )
    };

    let relative_root = {
        let base = PathBuf::from(&args.manifest_path).canonicalize().unwrap();
        let delta = base
            .strip_prefix(&cargo_project_root)
            .unwrap()
            .iter()
            .count();

        let res: String = (1..delta).into_iter().map(|_| "../").collect();

        res
    };

    println!("relative root: {:?}", relative_root);

    // println!("{:?}", cargo_dot_config_file);

    let mut config_file = OpenOptions::new()
        .create(true)
        .read(true)
        .write(true)
        .open(&cargo_dot_config_file)
        .unwrap();

    let mut config_contents = String::new();
    config_file.read_to_string(&mut config_contents).unwrap();

    let mut config_contents: toml::Table = toml::from_str(&config_contents).unwrap();

    match update_config_toml(
        &mut config_contents,
        &template_name,
        &debug_name,
        &deploy_name,
        toml_config_dir.to_str().unwrap(),
        generated_file.to_str().unwrap(),
        &relative_root,
    ) {
        Ok(_) => (),
        Err(e) => {
            log::error!("{}", e);
            return ExitCode::FAILURE;
        }
    }

    // writing env vars to config.toml
    let mut config_file = OpenOptions::new()
        .write(true)
        .truncate(true)
        .open(&cargo_dot_config_file)
        .unwrap();

    config_file
        .write_all(toml::to_string_pretty(&config_contents).unwrap().as_bytes())
        .unwrap();

    // create files with boilerplate
    match create_config_toml_files(
        &cargo_project_root,
        &toml_config_dir,
        &template_name,
        &debug_name,
        &deploy_name,
    ) {
        Ok(_) => (),
        Err(e) => {
            log::error!("Failed to create toml config files: {}", e);
            return ExitCode::FAILURE;
        }
    };

    // add rules to gitignore
    match update_gitignore_file(
        &cargo_project_root,
        toml_config_dir.to_str().unwrap(),
        &template_name,
        generated_file.to_str().unwrap(),
    ) {
        Ok(_) => (),
        Err(e) => {
            log::error!("Unable to update .gitignore: {}", e);
            return ExitCode::FAILURE;
        }
    }

    ExitCode::SUCCESS
}

/// Update the .cargo/config.toml table
fn update_config_toml(
    toml: &mut toml::Table,
    template: &str,
    debug: &str,
    deploy: &str,
    config_path: &str,
    generated_path: &str,
    relative_root: &str,
) -> Result<(), String> {
    let actual_config_path = format!("{}{}", relative_root, config_path);

    match toml.get_mut("env") {
        Some(e) => {
            if let Value::Table(t) = e {
                insert_into_env(
                    t,
                    template,
                    debug,
                    deploy,
                    &actual_config_path,
                    generated_path,
                );
            } else {
                return Err(format!("key \"env\" not defined as a table"));
            }
        }
        None => {
            let mut env_table = toml::Table::new();
            insert_into_env(
                &mut env_table,
                template,
                debug,
                deploy,
                &actual_config_path,
                generated_path,
            );
            toml.insert("env".to_string(), Value::Table(env_table));
        }
    }

    Ok(())
}

/// Used by `update_config_toml()`
fn insert_into_env(
    env_table: &mut toml::Table,
    template: &str,
    debug: &str,
    deploy: &str,
    config_path: &str,
    generated_path: &str,
) {
    env_table.insert(TEMPLATE_ENV.to_string(), Value::String(template.to_owned()));
    env_table.insert(DEBUG_ENV.to_string(), Value::String(debug.to_owned()));
    env_table.insert(DEPLOY_ENV.to_string(), Value::String(deploy.to_owned()));
    env_table.insert(
        CONFIG_PATH_ENV.to_string(),
        Value::String(config_path.to_owned()),
    );
    env_table.insert(
        GENERATED_FILE_PATH_ENV.to_string(),
        Value::String(generated_path.to_owned()),
    );
}

/// Creates the boilerplate toml config files that will be used for codegen
fn create_config_toml_files(
    project_root: &PathBuf,
    config_path: &PathBuf,
    template: &str,
    debug: &str,
    deploy: &str,
) -> Result<(), String> {
    fs::create_dir_all({
        let mut root = project_root.clone();
        root.push(config_path);
        root
    })
    .unwrap();

    let paths = [template, debug, deploy];

    for path in paths {
        let mut new_path = project_root.clone();
        new_path.push(config_path);
        new_path.push(path);

        // println  !("new path: {:?}", new_path);

        let mut file = OpenOptions::new()
            .create(true)
            .write(true)
            .read(true)
            .open(&new_path)
            .unwrap();

        let mut contents = String::new();
        let contents_len = file.read_to_string(&mut contents).unwrap();

        if contents_len != 0 {
            return Err("Config files already exist".to_string());
        }

        file.write(CONFIG_TOML_BOILERPLATE.as_bytes()).unwrap();
    }

    Ok(())
}

/// Create or update the gitignore file with new rules
fn update_gitignore_file(
    project_root: &PathBuf,
    config_path: &str,
    template: &str,
    generated_path: &str,
) -> Result<(), String> {
    const GITIGNORE: &'static str = ".gitignore";

    let root_rules = format!(
        "\n\n# added by {}\n{}\n",
        env!("CARGO_PKG_NAME"),
        generated_path,
    );

    let mut path = project_root.clone();
    path.push(GITIGNORE);

    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .unwrap();

    file.write(root_rules.as_bytes())
        .map_err(|e| e.to_string())?;

    let config_rules = format!(
        "# added by {}\n*.toml\n!{}",
        env!("CARGO_PKG_NAME"),
        template
    );

    let mut path = project_root.clone();
    path.push(config_path);
    path.push(GITIGNORE);

    let mut file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(path)
        .unwrap();

    file.write(config_rules.as_bytes())
        .map_err(|e| e.to_string())?;

    Ok(())
}
