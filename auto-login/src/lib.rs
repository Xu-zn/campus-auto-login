mod utils;
pub use utils::*;

use auto_login_common::config::ConfigFile;

use std::sync::OnceLock;

pub static CONFIG: OnceLock<ConfigFile> = OnceLock::new();