use reqwest::Client;
use std::{
    env::current_dir, time::Duration
};
use tokio_util::sync::CancellationToken;
use tklog::{error, info, warn};
use tokio::{
    task::JoinHandle,
    process::Command,
};
use crate::{
    config::ConfigFile,
    net::{
        check::check_connection, 
        login::{to_login, ChromeDriverConfig}
    },
};

use thirtyfour::{ ChromiumLikeCapabilities, DesiredCapabilities, WebDriver };

pub fn task_netcheck(config: ConfigFile, cancel_token: CancellationToken) -> JoinHandle<()> {
    let mut netcheck_interval = tokio::time::interval(Duration::from_secs(config.connection_wait));
    let netcheck = tokio::spawn(
        async move {
            let client = Client::new();
            loop {
                tokio::select! {
                    _ = cancel_token.cancelled() => {
                        info!("停止网络检测");
                        break;
                    },
                    _ = netcheck_interval.tick() => {
                        let connected = check_connection(&config.connection, &client).await;
                        if connected.is_err() {
    
                            net_connect(&config).await;
                        }
                        else {
                            info!("网络已连接");
                        }
                    }
                }
            }
            
        }
    );

    netcheck
}


pub fn task_stop() -> JoinHandle<()> {
    tokio::spawn(async {
        let options = ipmb::Options::new(
            "campus.auto-login", ipmb::label!("serve"), ""
        );
        let (sender, mut receiver) = match ipmb::join::<bool, String>(
            options, None
        ){
            Ok(t) => t,
            Err(_) => {
                error!("这里的问题就好像你要给心上人打个电话，然后信号连接的基站坏了");
                return;
            }
        };
    
        while let Ok(message) = receiver.recv(None) {
            match message.payload.as_str() {
                "live" => {
                    let selector = ipmb::Selector::unicast("manager");
                    let sender_message = ipmb::Message::new(selector, true);
                    sender.send(sender_message).unwrap_or_else(|_| {
                        warn!("给前任发了个消息，但对方死了，消息发送不过去了");
                    });
    
                },
                "exit" => {
                    break;
                },
                _ => {
                    warn!("收到了垃圾短信，也只能无视它罢了");
                }
            }
        };

    })
    
}


async fn net_connect(config: &ConfigFile) {
    let driver_info = &config.webdriver;

    let chrome_path = current_dir().unwrap().join(&driver_info.chrome_path).join("chrome.exe");
    if !chrome_path.exists() {
        error!("chrome文件不存在，总不能要求我做一个浏览器出来");
        return;
    }
    let chrome_path = chrome_path.as_os_str().to_str().unwrap();

    let driver_path = current_dir().unwrap().join(&driver_info.driver_path);
    if !driver_path.exists() {
        error!("driver文件不存在，就像给你一碗饭但不给筷子一样");
        return;
    }
    let driver_path = driver_path.as_os_str().to_str().unwrap();

    let driver_config = ChromeDriverConfig::default();
    let mut driver_command = Command::new(driver_path);
    driver_command
        .creation_flags(0x08000000)
        .args(&driver_config.to_args())
        .kill_on_drop(true);

    let driver_start = driver_command.spawn();
    if driver_start.is_err() {
        error!("启动chromedriver失败，总有刁民想害朕");
        return;
    } else {
        info!("启动chromedriver成功");
    }

    let driver_url = driver_config.get_driver_url();
    let mut chrome_capabilities = DesiredCapabilities::chrome();
    chrome_capabilities.set_binary(chrome_path).unwrap();
    chrome_capabilities.set_ignore_certificate_errors().unwrap();
    chrome_capabilities.set_no_sandbox().unwrap();
    chrome_capabilities.set_disable_gpu().unwrap();
    chrome_capabilities.add_arg("--headless=new").unwrap();

    let driver_client = WebDriver::new(&driver_url, chrome_capabilities).await;
    if let Err(_) = driver_client {
        error!("连接WebDriver失败，筷子断了");
        return;
    }

    let mut driver = driver_client.unwrap();
    let login_result = to_login(&config.login, &mut driver).await;
    if login_result.is_ok() {
        info!("登录成功");
    } else {
        error!("总会出现一些莫名其妙的问题，例如：\n{}", login_result.err().unwrap());
    }

    driver.quit().await.unwrap();
    drop(driver_command);
}