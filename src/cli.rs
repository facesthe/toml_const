use clap::Parser;

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

    /// Configuration dir for toml files, relative to $CARGO_MANIFEST_DIR
    #[clap(short, long, default_value = ".config/")]
    pub config_path: String,

    /// Path to generated file, relative to $CARGO_MANIFEST_DIR
    #[clap(short, long, default_value = "generated.rs")]
    pub generated_file_path: String,
}
