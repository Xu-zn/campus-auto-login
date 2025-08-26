
use std::time::Duration;
use thirtyfour::{By, WebDriver, extensions::query::ElementQueryable};
use tklog::warn;
use anyhow::{Result, anyhow};

use auto_login_common::config::Login;
use crate::utils::functional::sleep_millisecond;




pub async fn to_login(login: &Login, driver: &mut WebDriver) -> Result<()> {
    
    // 打开登陆页面
    let open_url = driver.goto(&login.config.eportal).await;

    // 打开页面失败时
    if let Err(e) = open_url {
        let error_msg = format!("网页打开失败: {}", e);
        return Err(anyhow!(error_msg));
    }

    // 如果是已经登陆的状态，浏览器会重定向到http://eportal.hhu.edu.cn/eportal/success.jsp
    // 此时检测url中是否存在"success"字符串即可
    let current_url = driver.current_url().await;
    // 获取current_url失败的话，默认为未登录情况
    if current_url.is_ok() {
        let url_query = current_url.unwrap();
        if let Some(query) = url_query.query() {
            if query.contains("success") {
                return Ok(());
            }
        }
    }
    
    // 检测登陆按钮，检测到登录按钮时，认为页面加载完成，超时10s
    let login_button = driver
        .query(By::Id("loginLink_div"))
        .wait(Duration::from_secs(login.config.timout), Duration::from_millis(200))
        .first().await;

    // 超时返回
    if login_button.is_err() {
        warn!("登陆页面加载超时，默认为已登录");
        return Ok(())
    }

    let login_button = login_button.unwrap();

    // 用户名
    let input_username = driver.find(By::Id("username")).await?;
    sleep_millisecond(100).await;

    input_username.click().await?;
    input_username.send_keys(&login.info.username).await?;


    // 密码
    let input_password = driver.find(By::Id("pwd")).await?;
    sleep_millisecond(100).await;
    driver.query(By::Id("pwd_tip")).first().await?.click().await?;
    input_password.send_keys(&login.info.password).await?;

    // 网络服务提供商
    let _ = driver.find(By::Id("selectDisname")).await?;
    sleep_millisecond(100).await;
    let selector = login.info.service.to_service();
    driver.find(By::Id(selector)).await?.click().await?;

    // 点击登录按钮
    login_button.click().await?;

    driver.close_window().await?;

    Ok(())
}
