#![allow(unused)]

#[derive(PartialEq)]
pub enum NetStatus {
    Connected,
    Restricted,
    Disconnected,
}

impl NetStatus {
    pub fn display(self: &Self) -> &'static str {
        match self {
            NetStatus::Connected => "已连接",
            NetStatus::Restricted => "网络受限",
            NetStatus::Disconnected => "未连接",
        }
    }
}