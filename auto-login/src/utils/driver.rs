use tokio::process::Command;
use std::path::PathBuf;
use anyhow::Result;
use thirtyfour::{ChromiumLikeCapabilities, DesiredCapabilities, WebDriver};
use auto_login_common::config::ChromeConfig;

pub struct ChromeOperator (pub ChromeConfig);

impl ChromeOperator {
    pub fn from_config(config: ChromeConfig) -> Self {
        Self(config)
    }

    pub fn get_args(self: &Self) -> Vec<String> {
        vec![
            format!("--port={}", self.0.port),
            format!("--silent=false")
        ]
    }

    pub fn get_driver_url(self: &Self) -> String {
        format!("http://127.0.0.1:{}", self.0.port)
    }

    /// 启动 ChromeDriver 进程
    pub fn start_chromedriver(self: &mut Self) -> Result<Command> {
        let mut driver_path = PathBuf::from(self.0.driver_path.as_str());
        if !driver_path.is_absolute() {
            let current = std::env::current_dir()?;
            driver_path = current.join(&driver_path);
        }

        let mut cmd = Command::new(&driver_path);
        #[cfg(windows)]
        cmd.creation_flags(0x08000000);
        cmd.args(self.get_args());
        let _child = cmd.spawn()?;

        Ok(cmd)
    }

    pub async fn start_chrome(self: &mut Self) -> Result<WebDriver> {
        let driver_url = self.get_driver_url();
        let mut chrome_path = PathBuf::from(self.0.browser_path.as_str());
        if !chrome_path.is_absolute() {
            let current = std::env::current_dir()?;
            chrome_path = current.join(&chrome_path);
        }
        println!("chrome path: {}", &chrome_path.display());

        let chrome_path = chrome_path.as_os_str().to_str().unwrap();

        let mut chrome_capabilities = DesiredCapabilities::chrome();
        chrome_capabilities.set_binary(chrome_path)?;
        chrome_capabilities.set_no_sandbox()?;
        chrome_capabilities.set_disable_gpu()?;
        chrome_capabilities.add_arg("--disable-ipv6")?;
        chrome_capabilities.add_arg("--headless=new")?;
        chrome_capabilities.add_arg("--ignore-certificate-errors")?;

        let webdriver = WebDriver::new(&driver_url, chrome_capabilities).await?;


        Ok(webdriver)
    }
}