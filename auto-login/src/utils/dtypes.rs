use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Clone)]
#[serde(rename="login")]
pub struct LoginInfo {
    pub username: String,
    pub password: String,
    pub service: NetServiceType,
    pub wait_seconds: u16,
}

#[derive(Deserialize, Serialize, Clone)]
#[serde(rename="connection")]
pub struct ConnectionState {
    pub url: String,
    pub value: String,
}

#[derive(Deserialize, Serialize, Clone)]
#[serde(rename="webdriver")]
pub struct WebDriverInfo {
    pub chrome_path: String,
    pub driver_path: String,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct ConfigFile {
    pub login: LoginInfo,
    pub connection: Vec<ConnectionState>,
    pub connection_wait: u64,
    pub webdriver: WebDriverInfo,
}

#[derive(Deserialize, Serialize, Clone)]
pub enum NetServiceType {
    #[serde(rename = "校园网")]
    CampusNet,
    #[serde(rename = "南京移动")]
    NanjingMobile,
    #[serde(rename = "常州电信")]
    ChangzhouTelecom,
    #[serde(rename = "常州联通")]
    ChangzhouUnion,
}



impl Default for ConfigFile{
    fn default() -> Self {
        Self {
            login: LoginInfo {
                username: "".into(),
                password: "".into(),
                service: NetServiceType::CampusNet,
                wait_seconds: 3,
            },
            connection: vec![ConnectionState {
                url: "http://www.msftncsi.com/ncsi.txt".into(),
                value: "Microsoft NCSI".into(),
            }],
            connection_wait: 30,
            webdriver: WebDriverInfo {
                chrome_path: "chrome-win64".into(),
                driver_path: "chromedriver.exe".into(),
            },
        }
    }
}
pub type ResultAsync = Box<dyn std::error::Error + Send + Sync>;

#[derive(PartialEq)]
pub enum NetStatus {
    Connecting,
    Restricted,
    Disconnected,
}