use super::dtypes::{ConfigFile, NetServiceType};
use std::env::current_dir;
use std::fs::{File, write as fs_write};
use tklog::{error, info};

impl ConfigFile {
    pub fn validation(self: &Self) -> Result<(), String> {
        let mut err_message = Vec::<String>::new();
        if self.connection_wait == 0 {
            err_message.push("连接等待时间需要大于0s".into())
        }
        if self.connection.len() == 0 {
            err_message.push("至少需要一个网络验证服务".into())
        }
        if self.login.username.len() == 0 {
            err_message.push("用户名不能为空".into())
        }
        if self.login.password.len() == 0 {
            err_message.push("密码不能为空".into())
        }
        if self.login.wait_seconds < 1 {
            err_message.push("等待时间不能小于1s".into())
        }

        if self.webdriver.chrome_path.len() == 0 {
            err_message.push("Chrome目录路径不能为空".into())
        }
        if self.webdriver.driver_path.len() == 0 {
            err_message.push("ChromeDriver目录路径不能为空".into())
        }

        if err_message.len() > 0 {
            return Err(err_message.join("\n\t"))
        }

        Ok(())
    }
    pub fn create_config_file(self: &Self) {
        let config_file_path  = current_dir().unwrap().join("../../../config.example.toml");
        if !config_file_path.exists() {
            let config_file = toml::to_string_pretty(&self).unwrap();
            fs_write("../../../config.example.toml", config_file).unwrap();
        }
    }
    pub fn load_config() -> Result<Self, ()> {
        let config_file_path  = current_dir().unwrap().join("../../../config.example.toml");
        let config_file = match File::open(&config_file_path) {
            Ok(file) => file,
            Err(_) => {
                error!("配置文件不存在，创建配置文件...");
                let default_config = ConfigFile::default();
                default_config.create_config_file();
                return Err(());
            }
        };

        // 读取配置文件
        let config_content = match std::io::read_to_string(config_file) {
            Ok(content) => content,
            Err(_) => {
                error!("读取配置文件失败");
                return Err(());
            }
        };

        // 解析配置文件
        let config = match toml::from_str::<ConfigFile>(&config_content) {
            Ok(config) => config,
            Err(_) => {
                error!("解析配置文件出错，请检查配置文件");
                return Err(());
            }
        };

        if let Err(err_message) = config.validation() {
            error!("配置文件验证失败: \n\t", err_message);
            return Err(());
        }
        info!("配置加载完成");
        Ok(config)
    }
}


impl NetServiceType {
    pub fn to_service_string(self: &Self) -> String {
        match self {
            NetServiceType::CampusNet => "_service_0".into(),
            NetServiceType::NanjingMobile => "_service_1".into(),
            NetServiceType::ChangzhouTelecom => "_service_2".into(),
            NetServiceType::ChangzhouUnion => "_service_3".into(),
        }
    }
}