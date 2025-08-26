use std::time::Duration;
use tklog::{error, info, warn};
use tokio::{task::JoinHandle, time::interval};
use tokio_util::sync::CancellationToken;
use crate::utils::driver::ChromeOperator;
use crate::utils::login::to_login;
use crate::CONFIG;

use auto_login_common::{ detect::detect_network_status, status::NetStatus };

const SERVICE_NAME: &str = "campus.auto-login";
const LABEL_NAME: &str = "serve";

/// 循环检测网络连通性
pub fn task_detection(cancel_token: CancellationToken) -> JoinHandle<()> {
    let query = &CONFIG.get().unwrap().query;
    tokio::spawn(async move {
        let mut detect_interval = interval(Duration::from_secs(query.interval));
        loop {
            tokio::select! {
                // 优先处理取消信号
                _ = cancel_token.cancelled() => {
                    info!("停止网络检测");
                    return;
                }
                // 定时检查网络状态
                _ = detect_interval.tick() => {
                    let cur_status = detect_network_status(query).unwrap();
                    println!("{}", cur_status.display());
                    match cur_status {
                        NetStatus::Connected => info!("网络已连接"),
                        NetStatus::Restricted => warn!("受限网络"),
                        NetStatus::Disconnected => net_connect().await,
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

async fn net_connect() {
    let config = CONFIG.get().unwrap();
    let driver_config = &config.driver.chrome_config;
    let mut chrome = ChromeOperator(driver_config.clone().unwrap());

    let driver_command = match chrome.start_chromedriver() {
        Ok(t) => t,
        Err(e) => {
            error!("启动ChromeDriver失败：\n", e);
            return;
        }
    };
    let mut driver_client = match chrome.start_chrome().await {
        Ok(t) => t,
        Err(e) => {
            error!("启动Chrome失败：\n", e);
            return;
        }
    };

    let login_result = to_login(&config.login, &mut driver_client).await;
    match login_result  {
        Ok(_) => info!("登录成功"),
        Err(e) => error!("登录失败：\n", e),
    }

    match driver_client.quit().await {
        Ok(_) => {},
        Err(e) => error!("关闭Chrome失败：\n", e),
    };
    drop(driver_command);
}
