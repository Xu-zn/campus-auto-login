use serde::{Deserialize, Serialize};


#[derive(Deserialize, Serialize, Clone)]
#[serde(rename="login")]
pub struct LoginInfo {
    pub username: String,
    pub password: String,
    pub service: String,
    pub wait_seconds: u16,
    pub eportal: String
}


#[derive(Deserialize, Serialize, Clone)]
#[serde(rename="connection")]
pub struct ConnectionState {
    pub url: String,
    pub value: String,
}


// #[derive(Deserialize, Serialize)]
// #[serde(rename="wlan")]
// pub struct WlanInfo{
//     pub ssid: String,
//     pub password: String,
// }


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

pub type ResultAsync = Box<dyn std::error::Error + Send + Sync>;