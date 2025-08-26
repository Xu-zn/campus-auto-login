use clap::Parser;
use std::{env::current_dir, fs};
// use clap::Parser;
use manager_cli::utils::{
    args::{Cli, Commands},
    download::{UseChrome, download_file},
    extract::extract_file,
    platform::detect_platform,
};

use auto_login_common::config::ConfigFile;

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Download(args) => {
            // 检测平台
            let platform = detect_platform();
            let platform = platform.to_string();

            // 下载后的文件名
            let chrome_name = format!("chrome-{}.zip", platform);
            let current_dir = std::env::current_dir().unwrap();
            let chrome_dest = current_dir.join(chrome_name);
            // 下载 Chrome 浏览器
            let chrome_url = UseChrome::Chrome.generate_download_url(&platform);
            println!("下载Chrome for Test");
            download_file(&chrome_url, &chrome_dest).await.unwrap();
            // 解压缩
            extract_file(&chrome_dest, &current_dir).unwrap();

            let driver_name = format!("chromedriver-{}.zip", platform);
            // let current_dir = std::env::current_dir().unwrap();
            let driver_dest = current_dir.join(driver_name);
            // 下载 Chrome 浏览器
            let chrome_url = UseChrome::ChromeDriver.generate_download_url(&platform);
            println!("下载ChromeDriver for Test");
            download_file(&chrome_url, &driver_dest).await.unwrap();
            // 解压缩
            extract_file(&driver_dest, &current_dir).unwrap();

            if args.delete {
                println!("删除Zip文件");
                fs::remove_file(&chrome_dest).unwrap();
                fs::remove_file(&driver_dest).unwrap();
            }
        }
        Commands::Config(args) => {
            if args.create {
                let current_dir = current_dir().unwrap();
                let target_dir = current_dir.join("config.toml");
                if target_dir.exists() {
                    println!("配置文件已存在");
                } else {
                    ConfigFile::create_default_config(&current_dir).unwrap();
                }
            } else if args.validate {
                // 调用验证配置文件的函数
                // validate_config();
                println!("验证配置文件，但未实现");
            }
            // 可以添加 else 处理未指定参数的情况
        }
        Commands::Status => {
            // 调用查看状态的函数
            // show_status();
            println!("查看状态");
        }
    }
}
