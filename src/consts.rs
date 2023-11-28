//! Consts and keys for env vars

pub const TEMPLATE_ENV: &'static str = "TOML_CONST_TEMPLATE";

pub const DEBUG_ENV: &'static str = "TOML_CONST_DEBUG";

pub const DEPLOY_ENV: &'static str = "TOML_CONST_DEPLOY";

pub const CONFIG_PATH_ENV: &'static str = "TOML_CONST_CONFIG_PATH";

pub const GENERATED_FILE_PATH_ENV: &'static str = "TOML_CONST_GENERATED_PATH";

/// Relative path from indicated manifest to root manifest.
///
/// A root manifest may exist if it is a workspace, or a parent package of
/// the current manifest.
pub const ROOT_MANIFEST_RELATIVE_PATH: &'static str = "TOML_CONST_ROOT_MANIFEST_RELATIVE_PATH";

pub const CONFIG_TOML_BOILERPLATE: &'static str =
    "# this key MUST be present in all configuration files
use = false

# add your config key-value pairs below:
";
