mod utils;
pub use utils::*;

use campus_core::config::ConfigFile;
use campus_core::elements::Elements;

use std::sync::OnceLock;

pub static CONFIG: OnceLock<ConfigFile> = OnceLock::new();
pub static ELEMENTS: OnceLock<Elements> = OnceLock::new();
/// auto-login 启动时的 Unix 时间戳（秒），供 ipmb uptime 查询使用
pub static STARTUP_TIMESTAMP: OnceLock<u64> = OnceLock::new();