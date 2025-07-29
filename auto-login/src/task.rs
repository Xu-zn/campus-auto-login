use crate::{
    constants::GLOBAL_NET_STATUS,
    dtypes::{ConfigFile, NetStatus},
    networking::{
        check::check_connection,
        login::{to_login, ChromeDriverConfig},
    },
};
use reqwest::Client;
use std::{env::current_dir, time::Duration};
use thirtyfour::{ChromiumLikeCapabilities, DesiredCapabilities, WebDriver};
use tklog::{error, info, warn};
use tokio::{process::Command, task::JoinHandle, time::interval};
use tokio_util::sync::CancellationToken;

const SERVICE_NAME: &str = "campus.auto-login";
const LABEL_NAME: &str = "serve";

/// 循环检测网络连通性
pub fn task_net_check(config: ConfigFile, cancel_token: CancellationToken) -> JoinHandle<()> {
    tokio::spawn(async move {
        let mut check_interval = interval(Duration::from_secs(config.connection_wait));
        let client = Client::new();

        let mut last_status = NetStatus::Disconnected;

        loop {
            tokio::select! {
                // 优先处理取消信号
                _ = cancel_token.cancelled() => {
                    info!("停止网络检测");
                    return;
                }
                // 定时检查网络状态
                _ = check_interval.tick() => {
                    let cur_status = check_connection(&config.connection, &client).await;
                    match cur_status {
                        NetStatus::Connecting => info!("网络已连接"),
                        NetStatus::Restricted => warn!("受限网络"),
                        NetStatus::Disconnected => net_connect(&config).await,
                    }
                    if cur_status != last_status {
                        last_status = cur_status;
                        {
                            let mut write_guard = GLOBAL_NET_STATUS.write().unwrap();
                            *write_guard = NetStatus::Connecting;
                        }
                    }

                }

            }
        }
    })
}

pub fn task_stop() -> JoinHandle<()> {
    tokio::spawn(async {
        let options = ipmb::Options::new(SERVICE_NAME, ipmb::label!(LABEL_NAME), "");
        let (_, mut receiver) = match ipmb::join::<(), String>(options, None) {
            Ok(t) => t,
            Err(_) => {
                error!("ipmb连接失败");
                return;
            }
        };

        while let Ok(message) = receiver.recv(None) {
            match message.payload.as_str() {
                "exit" => {
                    break;
                }
                _ => { /*不想响应来历不明的信号*/ }
            }
        }
    })
}

async fn net_connect(config: &ConfigFile) {
    let driver_info = &config.webdriver;

    let chrome_path = current_dir()
        .unwrap()
        .join(&driver_info.chrome_path)
        .join("chrome.exe");
    if !chrome_path.exists() {
        error!("未检测到chrome");
        return;
    }
    let chrome_path = chrome_path.as_os_str().to_str().unwrap();

    let driver_path = current_dir().unwrap().join(&driver_info.driver_path);
    if !driver_path.exists() {
        error!("未检测到chromedriver");
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
        error!("启动chromedriver失败");
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
        error!("连接WebDriver失败");
        return;
    }

    let mut driver = driver_client.unwrap();
    let login_result = to_login(&config.login, &mut driver).await;
    if login_result.is_ok() {
        info!("登录成功");
    } else {
        error!("登录出现意外：\n", login_result.unwrap_err());
    }
    driver.quit().await.unwrap();
    drop(driver_command);
}
