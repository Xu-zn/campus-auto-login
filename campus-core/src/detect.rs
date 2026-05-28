use reqwest::Client;
use crate::config::QueryConfig;
use crate::errors::CampusError;
use crate::status::NetStatus;

/// 探测网络连通性状态
///
/// 依次请求配置中的检测 URL，根据响应判断网络状态：
/// - 任意 URL 响应内容匹配预期值 → `Connected`
/// - 任意 URL 返回非 2xx 状态码 → `Restricted`（被门户拦截）
/// - 所有 URL 请求失败或内容不匹配 → `Disconnected`
pub async fn detect_network_status(query: &QueryConfig, client: &Client) -> Result<NetStatus, CampusError> {
    for conn in &query.connect {
        // 请求失败 → 尝试下一个 URL
        let resp = match client.get(&conn.url).send().await {
            Ok(r) => r,
            Err(_) => continue,
        };

        // 非 2xx → 被门户重定向，网络受限
        if !resp.status().is_success() {
            return Ok(NetStatus::Restricted);
        }

        // 响应内容匹配 → 网络已连通
        let text = resp.text().await
            .map_err(CampusError::NetworkRequest)?;
        if text == conn.val {
            return Ok(NetStatus::Connected);
        }
        // 内容不匹配 → 继续尝试下一个 URL
    }

    Ok(NetStatus::Disconnected)
}
