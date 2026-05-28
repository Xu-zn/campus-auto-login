use std::time::Duration;
use thirtyfour::{By, WebDriver, extensions::query::ElementQueryable};
use tklog::warn;

use crate::utils::functional::sleep_millisecond;
use campus_core::config::Login;
use campus_core::elements::Elements;
use campus_core::errors::CampusError;

/// 页面元素操作之间的等待间隔（毫秒）
const CLICK_INTERVAL_MS: u64 = 100;

/// 点击 tip 元素展开输入区域，然后向目标输入框填入值
async fn click_tip_and_fill(
    driver: &WebDriver,
    tip_id: &str,
    input_id: &str,
    value: &str,
) -> Result<(), CampusError> {
    driver
        .find(By::Id(tip_id))
        .await
        .map_err(|e| CampusError::LoginFailed(format!("查找占位元素失败: {e}")))?
        .click()
        .await
        .map_err(|e| CampusError::LoginFailed(format!("点击占位元素失败: {e}")))?;

    sleep_millisecond(CLICK_INTERVAL_MS).await;

    driver
        .find(By::Id(input_id))
        .await
        .map_err(|e| CampusError::LoginFailed(format!("查找输入框失败: {e}")))?
        .send_keys(value)
        .await
        .map_err(|e| CampusError::LoginFailed(format!("输入失败: {e}")))?;

    sleep_millisecond(CLICK_INTERVAL_MS).await;

    Ok(())
}

pub async fn to_login(
    login: &Login,
    driver: &mut WebDriver,
    elements: &Elements,
) -> Result<(), CampusError> {
    // 打开登陆页面
    driver
        .goto(&login.config.eportal)
        .await
        .map_err(|e| CampusError::LoginFailed(format!("打开登陆页面失败: {e}")))?;

    // 如果是已经登陆的状态，浏览器会重定向到 success.jsp
    // 此时检测 url 中是否存在 "success" 字符串即可
    if let Ok(current_url) = driver.current_url().await {
        if let Some(query) = current_url.query() {
            if query.contains("success") {
                return Ok(());
            }
        }
    }

    // 等待登录按钮出现（页面加载完成的标志），超时则假定已登录
    let login_button = driver
        .query(By::Id(&elements.page.login_button))
        .wait(
            Duration::from_secs(login.config.timout),
            Duration::from_millis(200),
        )
        .first()
        .await;

    let Ok(login_button) = login_button else {
        warn!("登陆页面加载超时，默认为已登录");
        return Ok(());
    };

    // 用户名
    click_tip_and_fill(
        driver,
        &elements.page.username_tip,
        &elements.page.username_input,
        &login.info.username,
    )
    .await?;

    // 密码
    click_tip_and_fill(
        driver,
        &elements.page.password_tip,
        &elements.page.password_input,
        &login.info.password,
    )
    .await?;

    // 网络服务提供商
    {
        // 点击下拉框 tip 展开选项
        driver
            .find(By::Id(&elements.page.service_tip))
            .await
            .map_err(|e| CampusError::LoginFailed(format!("查找服务商下拉框失败: {e}")))?
            .click()
            .await
            .map_err(|e| CampusError::LoginFailed(format!("点击服务商下拉框失败: {e}")))?;

        sleep_millisecond(CLICK_INTERVAL_MS).await;

        // 选择具体的服务选项（ID 由 config.toml 中 service 字段指定）
        driver
            .find(By::Id(&login.info.service))
            .await
            .map_err(|e| CampusError::LoginFailed(format!("查找服务选项失败: {e}")))?
            .click()
            .await
            .map_err(|e| CampusError::LoginFailed(format!("选择服务失败: {e}")))?;

        sleep_millisecond(CLICK_INTERVAL_MS).await;
    }

    // 点击登录按钮
    login_button
        .click()
        .await
        .map_err(|e| CampusError::LoginFailed(format!("点击登录按钮失败: {e}")))?;

    driver
        .close_window()
        .await
        .map_err(|e| CampusError::LoginFailed(format!("关闭浏览器窗口失败: {e}")))?;

    Ok(())
}
