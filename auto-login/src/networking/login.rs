use serde::{Deserialize, Serialize};
use std::time::Duration;
use thirtyfour::{By, WebDriver, extensions::query::ElementQueryable};
use tklog::warn;
use crate::constants::PORTAL;
use crate::utils::dtypes::{LoginInfo, ResultAsync};
use crate::sleep_millisecond;

/// ChromeDriver
#[derive(Serialize, Deserialize)]
pub struct ChromeDriverConfig {
    /// 使用端口
    pub port: u16,
    /// 记录日志
    pub silent: bool,
}

impl ChromeDriverConfig {
    pub fn default() -> Self {
        Self {
            port: 18888,
            silent: true,
        }
    }

    pub fn set_port(self: &mut Self, driver_port: u16) {
        self.port = driver_port
    }

    pub fn set_silent(self: &mut Self, driver_silent: bool) {
        self.silent = driver_silent
    }

    pub fn get_driver_url(self: &Self) -> String {
        format!("http://127.0.0.1:{}", self.port)
    }

    pub fn to_args(self: &Self) -> Vec<String> {
        let mut arg: Vec<String> = Vec::new();
        arg.push(format!("--port={}", self.port));
        if self.silent {
            arg.push("--silent".into());
        }
        arg
    }
}



pub async fn to_login(login_info: &LoginInfo, driver: &mut WebDriver) -> Result<(), ResultAsync> {
    
    // 打开登陆页面
    let open_url = driver.goto(PORTAL).await;

    // 打开页面失败时
    if let Err(e) = open_url {
        let error_msg = format!("网页打开失败: {}", e);
        return Err(error_msg.into());
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
    let login_button = driver.query(By::Id("loginLink_div"))
                                   .wait(Duration::from_secs(10), Duration::from_millis(500)).first().await;
    // 超时返回Error
    if login_button.is_err() {
        warn!("登陆页面加载超时，默认为已登录");
        return Ok(())
    }

    let login_button = login_button.unwrap();

    // 用户名
    let input_username = driver.find(By::Id("username")).await;
    sleep_millisecond(200).await;
    match input_username {
        Ok(el) => {
            el.click().await?;
            el.send_keys(&login_info.username).await?;
        },
        Err(e) => {
            let error_msg = format!("用户名输入框查找失败: {}", e);
            return Err(error_msg.into());
        }
    }

    // 密码
    let input_password = driver.find(By::Id("pwd")).await;
    match input_password {
        Ok(el) => {
            sleep_millisecond(200).await;
            driver.query(By::Id("pwd_tip")).first().await?.click().await?;
            sleep_millisecond(200).await;
            el.send_keys(&login_info.password).await?;
        },
        Err(e) => {
            let error_msg = format!("密码输入框查找失败: {}", e);
            return Err(error_msg.into());
        }
    }

    // 网络服务提供商
    let input_service = driver.find(By::Id("selectDisname")).await;
    match input_service {
        Ok(el) => {
            // 先点一下，否则无法选择
            el.click().await?;
            println!("start find service");
            // 选择服务
            let selector = login_info.service.to_service_string();
            driver.find(By::Id(&selector)).await?.click().await?;
        },
        Err(e) => {
            let error_msg = format!("服务选择框查找失败: {}", e);   
            return Err(error_msg.into());
        }
    }

    // 点击登录按钮
    login_button.click().await?;

    driver.close_window().await?;

    Ok(())
}
