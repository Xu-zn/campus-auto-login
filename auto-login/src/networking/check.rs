use reqwest::Client;
use crate::dtypes::{ConnectionState, NetStatus};

/// 检测当前网络是否能访问外网
pub async fn check_connection(connection: &Vec<ConnectionState>, client: &Client) -> NetStatus {
    let mut status = NetStatus::Disconnected;

    for conn in connection {

        let res = client.get(&conn.url).send().await;
        if res.is_ok() {
            let res = res.unwrap();
            let res_status = res.status();
            let res_text = res.text().await.unwrap_or("restricted".to_string());
            if res_status.is_success() {
                if res_text.eq(&conn.value) {
                    status = NetStatus::Connecting;
                }
                else {
                    status = NetStatus::Restricted;
                }
            }
        }
    }
    status
}