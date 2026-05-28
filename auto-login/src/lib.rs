mod utils;
pub use utils::*;

use campus_core::config::ConfigFile;
use campus_core::elements::Elements;

use std::sync::OnceLock;

pub static CONFIG: OnceLock<ConfigFile> = OnceLock::new();
pub static ELEMENTS: OnceLock<Elements> = OnceLock::new();