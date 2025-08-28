use std::time::Duration;
use reqwest::Client;
use tklog::{error, info, warn};
use tokio::{task::JoinHandle, time::interval};


use tokio_util::sync::CancellationToken;
use crate::utils::driver::ChromeOperator;
use crate::utils::login::to_login;
use crate::CONFIG;

use auto_login_common::{ detect::detect_network_status, status::NetStatus };



/// 循环检测网络连通性
pub fn task_detection(cancel_token: CancellationToken) -> JoinHandle<()> {
    let query = &CONFIG.get().unwrap().query;
    let client = Client::new();
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
                    let cur_status = detect_network_status(query, &client).await.unwrap();
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

#[cfg(windows)]
pub fn task_stop() -> JoinHandle<()> {
    tokio::spawn(async {
        let options = ipmb::Options::new("campus-login", ipmb::label!("core"), "");
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

#[cfg(not(windows))]
pub fn task_stop() -> JoinHandle<()> {
    use tokio::io::{AsyncBufReadExt, BufReader};
    use tokio::net::UnixListener;
    use std::path::Path;
    let socket_path = "/tmp/campus_login.sock";
    if Path::new(socket_path).exists() {
        std::fs::remove_file(socket_path).ok();
    }
    let listener = UnixListener::bind(socket_path);
    let listener = match listener {
        Ok(t) => t,
        Err(e) => {
            error!("创建套接字失败：\n", e);
            return tokio::spawn(async {});
        }
    };
    tokio::spawn(async move {
        let (stream, _) = match listener.accept().await {
            Ok(conn) => conn,
            Err(e) => {
                error!("accept error: ", e);
                return;
            }
        };

        let mut lines = BufReader::new(stream).lines();
        while let Ok(Some(line)) = lines.next_line().await {
            if line == "exit" {
                info!("[core] 收到 exit，准备退出...");
                break;
            } else {
                info!("[core] 忽略消息: ", line);
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
            warn!("Chrome问题：\n", e);
            return;
        }
    };

    let login_result = to_login(&config.login, &mut driver_client).await;
    match login_result  {
        Ok(_) => info!("登录成功"),
        Err(e) => error!("登录失败，错误信息：\n", e),
    }

    match driver_client.quit().await {
        Ok(_) => {},
        Err(e) => error!("关闭Chrome失败：\n", e),
    };
    drop(driver_command);
}
