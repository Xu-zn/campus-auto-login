use std::time::Duration;
use reqwest::Client;
use tklog::{error, info, warn};
use tokio::{task::JoinHandle, time::interval};

use tokio_util::sync::CancellationToken;
use crate::utils::driver::ChromeOperator;
use crate::utils::login::to_login;
use crate::{CONFIG, ELEMENTS};

use campus_core::{ detect::detect_network_status, status::NetStatus };

/// 循环检测网络连通性
pub fn task_detection(cancel_token: CancellationToken) -> JoinHandle<()> {
    let config = match CONFIG.get() {
        Some(c) => c,
        None => {
            error!("检测任务启动失败: 配置未加载");
            return tokio::spawn(async {});
        }
    };
    let interval_secs = config.query.interval;
    let query = config.query.clone();
    let client = Client::new();

    tokio::spawn(async move {
        let mut detect_interval = interval(Duration::from_secs(interval_secs));
        loop {
            tokio::select! {
                _ = cancel_token.cancelled() => {
                    info!("停止网络检测");
                    return;
                }
                _ = detect_interval.tick() => {
                    let cur_status = match detect_network_status(&query, &client).await {
                        Ok(s) => s,
                        Err(e) => {
                            error!("网络检测异常: ", e);
                            continue;
                        }
                    };
                    match cur_status {
                        NetStatus::Connected => info!("网络已连接"),
                        NetStatus::Restricted => {
                            warn!("受限网络");
                            net_connect().await;
                            // 重置 interval 避免 net_connect 耗时过长
                            // 导致积压 tick 在返回后立即触发二次连接
                            detect_interval.reset();
                        }
                        NetStatus::Disconnected => {
                            net_connect().await;
                            detect_interval.reset();
                        }
                    }
                }
            }
        }
    })
}

#[cfg(windows)]
pub fn task_stop() -> JoinHandle<()> {
    use crate::STARTUP_TIMESTAMP;

    tokio::spawn(async {
        let options = ipmb::Options::new("campus-login", ipmb::label!("core"), "");
        let (sender, mut receiver) = match ipmb::join::<String, String>(options, None) {
            Ok(t) => t,
            Err(_) => {
                error!("ipmb连接失败");
                return;
            }
        };

        while let Ok(message) = receiver.recv(None) {
            match message.payload.as_str() {
                "exit" => break,
                "uptime" => {
                    let ts = STARTUP_TIMESTAMP
                        .get()
                        .map(|t| t.to_string())
                        .unwrap_or_else(|| "0".to_string());
                    let reply = ipmb::Message::new(
                        ipmb::Selector::unicast("cli"),
                        ts,
                    );
                    let _ = sender.send(reply);
                }
                _ => { /* 忽略来历不明的信号 */ }
            }
        }
    })
}

#[cfg(not(windows))]
pub fn task_stop() -> JoinHandle<()> {
    use tokio::net::UnixListener;
    use std::path::Path;

    let socket_path = "/tmp/campus_login.sock";
    if Path::new(socket_path).exists() {
        if let Err(e) = std::fs::remove_file(socket_path) {
            warn!("清理残留 socket 失败: ", e);
        }
    }

    let listener = match UnixListener::bind(socket_path) {
        Ok(t) => t,
        Err(e) => {
            error!("创建套接字失败: ", e);
            return tokio::spawn(async {});
        }
    };

    tokio::spawn(async move {
        use tokio::io::{AsyncBufReadExt, BufReader};

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
            }
            info!("[core] 忽略消息: ", line);
        }
    })
}

async fn net_connect() {
    let config = match CONFIG.get() {
        Some(c) => c,
        None => {
            error!("登录失败: 配置未加载");
            return;
        }
    };
    let elements = match ELEMENTS.get() {
        Some(e) => e,
        None => {
            error!("登录失败: 页面元素配置未加载");
            return;
        }
    };

    let driver_config = match &config.driver.chrome_config {
        Some(c) => c.clone(),
        None => {
            error!("登录失败: Chrome 配置缺失");
            return;
        }
    };
    let mut chrome = ChromeOperator::from_config(driver_config);

    let driver_command = match chrome.start_chromedriver() {
        Ok(t) => t,
        Err(e) => {
            error!("启动ChromeDriver失败: ", e);
            return;
        }
    };
    let mut driver_client = match chrome.start_chrome().await {
        Ok(t) => t,
        Err(e) => {
            warn!("Chrome问题: ", e);
            return;
        }
    };

    match to_login(&config.login, &mut driver_client, elements).await {
        Ok(_) => info!("登录成功"),
        Err(e) => error!("登录失败，错误信息: ", e),
    }

    if let Err(e) = driver_client.quit().await {
        error!("关闭Chrome失败: ", e);
    }
    drop(driver_command);
}
