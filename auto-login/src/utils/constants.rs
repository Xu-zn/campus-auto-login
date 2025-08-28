use std::sync::{RwLock, LazyLock};
use super::dtypes::NetStatus;


pub const PORTAL: &'static str = "http://eportal.hhu.edu.cn/";

pub static GLOBAL_NET_STATUS: LazyLock<RwLock<NetStatus>> = LazyLock::new(|| RwLock::new(NetStatus::Disconnected));