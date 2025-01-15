use tokio::time::{sleep, Duration};
use notify_rust::{Notification, Timeout};

use crate::config::ConfigFile;
use tklog::{error, info};

use std::fs::File;


pub async fn sleep_secs(secs: u64) {
    sleep(Duration::from_secs(secs)).await;
}

pub async fn sleep_millisecs(millisecs: u64) {
    sleep(Duration::from_millis(millisecs)).await;
}




pub fn toast(msg: &str) {
    let tmp = Notification::new()
    .summary("AutoLogin")
    .body(msg)
    .timeout(Timeout::Milliseconds(6000)) //milliseconds
    .show();

    if tmp.is_err() {
        println!("{}", tmp.unwrap_err())
    }
}


pub fn init_config(curdir: std::path::PathBuf) -> Result<ConfigFile, &'static str> {
    // 从当前目录下加载配置文件`config.toml`
    let config_file_path = curdir.join("config.toml");
    let config_file = match File::open(&config_file_path) {
        Ok(file) => file,
        Err(_) => {
            let message = "配置文件不存在";
            error!(&message);
            toast(&message);
            return Err(message);
        }
    };
 
    // 读取配置文件
    let config_content = match std::io::read_to_string(config_file) {
        Ok(content) => content,
        Err(_) => {
            let message = "读取配置文件失败";
            error!(&message);
            toast(&message);
            return Err(message);
        }
    };

    // 解析配置文件
    let config = match toml::from_str::<ConfigFile>(&config_content) {
        Ok(config) => config,
        Err(_) => {
            let message ="解析配置文件出错，请检查配置文件";
            error!(&message);
            toast(&message);
            return Err(message);
        }
    };

    info!("配置加载完成");
    Ok(config)
}