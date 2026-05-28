use serde::{Deserialize, Serialize};
use std::{path::Path, fs::{write, read_to_string, create_dir_all}};
use crate::errors::CampusError;


#[derive(Deserialize, Serialize)]
pub enum DriverType {
    Gecko,
    Chrome,
}


/// 用户的登陆信息
#[derive(Deserialize, Serialize)]
#[serde(rename="info")]
pub struct LoginInfo {
    /// 用户名
    pub username: String,
    /// 密码
    pub password: String,
    /// 登录服务
    pub service: String,
}

impl LoginInfo {
    fn default() -> Self {
        Self { 
            username: String::new(), 
            password: String::new(), 
            service: String::from("_service_0"), 
        }
    }
}

/// 登陆配置信息
#[derive(Deserialize, Serialize)]
#[serde(rename="config")]
pub struct LoginConfig {
    /// 门户地址
    pub eportal: String,
    /// 页面加载的等待时间（秒）
    pub timout: u64,
}

impl LoginConfig {
    fn default() -> Self {
        Self {
            eportal: String::from("http://eportal.hhu.edu.cn"),
            timout: 3,
        }
    }
}

#[derive(Deserialize, Serialize)]
#[serde(rename="login")]
pub struct Login {
    pub info: LoginInfo,
    pub config: LoginConfig,
}

impl Login {
    fn default() -> Self {
        Self { 
            info: LoginInfo::default(), 
            config: LoginConfig::default(),
        }
    }
}

#[derive(Deserialize, Serialize, Clone)]
#[serde(rename="connection")]
pub struct Connection {
    pub url: String,
    #[serde(rename="value")]
    pub val: String,
}


#[derive(Deserialize, Serialize, Clone)]
#[serde(rename="query")]
pub struct QueryConfig {
    pub connect: Vec<Connection>,
    pub interval: u64,
}

impl QueryConfig {
    fn default() -> Self {
        Self { 
            connect: vec![
                Connection {
                    url: String::from("http://www.msftncsi.com/ncsi.txt"),
                    val: String::from("Microsoft NCSI")
                },

                Connection {
                    url: String::from("https://captive.apple.com/hotspot-detect.html"),
                    val: String::from("Success")
                }
                
            ], 
            interval: 15
        }
    }
}

#[derive(Deserialize, Serialize, Clone)]
#[serde(rename="chrome")]
pub struct ChromeConfig {
    pub port: u16,
    pub driver_path: String,
    pub browser_path: String,
}

impl ChromeConfig {
    pub fn default() -> Self {
        let platform = crate::platform::detect_platform();

        #[cfg(windows)]
        return Self {
            port: 18888,
            driver_path: format!("chromedriver-{}/chromedriver.exe", platform),
            browser_path: format!("chrome-{}/chrome.exe", platform),
        };
        #[cfg(not(windows))]
        return Self {
            port: 18888,
            driver_path: format!("chromedriver-{}/chromedriver", platform),
            browser_path: format!("chrome-{}/chrome", platform),
        };
    }
}

#[derive(Deserialize, Serialize, Clone)]
#[serde(rename="gecko")]
pub struct GeckoConfig {
    pub port: u16,
    pub driver_path: String,
    pub browser_path: String,
}

impl GeckoConfig {
    pub fn default() -> Self {
        Self {
            port: 18888,
            driver_path: String::from(""),
            browser_path: String::from(""),
        }
    }
}

#[derive(Deserialize, Serialize)]
#[serde(rename="driver")]
pub struct DriverConfig {
    pub driver_type: DriverType,
    #[serde(rename="chrome")]
    pub chrome_config: Option<ChromeConfig>,
    #[serde(rename="gecko")]
    pub firefox_config: Option<GeckoConfig>,
}

impl DriverConfig {
    pub fn default() -> Self {
        Self {
            driver_type: DriverType::Chrome,
            chrome_config: Some(ChromeConfig::default()),
            firefox_config: None,
        }
    }
}


#[derive(Deserialize, Serialize)]
pub struct ConfigFile {
    pub login: Login,
    pub query: QueryConfig,
    pub driver: DriverConfig,
}

impl ConfigFile {
    pub fn default() -> Self {
        Self { 
            login: Login::default(),
            query: QueryConfig::default(),
            driver: DriverConfig::default(),
        }
    }

    pub fn save_config(&self, target_dir: &Path) -> Result<(), CampusError> {
        if !target_dir.exists() {
            create_dir_all(target_dir)?;
        }
        let target_file = target_dir.join("config.toml");
        let config_text = toml::to_string_pretty(self)
            .map_err(CampusError::ConfigSerialize)?;
        write(&target_file, config_text)?;
        Ok(())
    }

    pub fn create_default_config(target_dir: &Path) -> Result<(), CampusError> {
        let default_config = Self::default();
        default_config.save_config(target_dir)
    }

    pub fn load_config(from_path: &Path) -> Result<Self, CampusError> {
        let text = match read_to_string(from_path) {
            Ok(t) => t,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                return Err(CampusError::ConfigNotFound(from_path.display().to_string()));
            }
            Err(e) => return Err(e.into()),
        };
        let config: ConfigFile = toml::from_str(&text)
            .map_err(CampusError::ConfigParse)?;
        Ok(config)
    }
}






