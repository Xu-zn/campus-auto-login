use anyhow::{Ok, Result};
use reqwest::Client;
use crate::config::QueryConfig;
use crate::status::NetStatus;


/// 探测网络连通性状态（同步）
pub async fn detect_network_status(query: &QueryConfig, client: &Client) -> Result<NetStatus> {
    let connections = &query.connect;

    for conn in connections {
        let resp = client.get(&conn.url).send().await;
        if resp.is_ok() {
            let resp = resp.unwrap();
            if resp.status().is_success() {
                let text = resp.text().await;
                if text.is_ok() {
                    let text = text.unwrap();
                    if text.contains(&conn.val) {
                        return Ok(NetStatus::Connected);
                    }
                }
            } else {
                return Ok(NetStatus::Restricted);
            }
        }
    }

    Ok(NetStatus::Disconnected)
}
