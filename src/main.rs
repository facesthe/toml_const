use std::process::ExitCode;

use toml_const::cli;

fn main() -> ExitCode {
    pretty_env_logger::init();
    cli::run()
}
