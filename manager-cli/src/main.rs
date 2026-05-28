use clap::Parser;
use std::{env::current_dir, fs};
use manager_cli::utils::{
    args::{Cli, Commands},
    download::{UseChrome, download_file},
    extract::extract_file,
    platform::detect_platform,
};

use campus_core::{config::ConfigFile, process};

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Download(args) => {
            let platform = detect_platform();
            let plat = args.platform.as_ref().unwrap_or(&platform);
            let current_dir = std::env::current_dir().unwrap();

            // 如果未指定 chrome/driver，则默认下载全部
            let dl_chrome = args.chrome || (!args.chrome && !args.driver);
            let dl_driver = args.driver || (!args.chrome && !args.driver);

            if dl_chrome {
                let chrome_name = format!("chrome-{}.zip", plat);
                let chrome_dest = current_dir.join(&chrome_name);
                println!("下载Chrome for Testing");
                let url = UseChrome::Chrome.generate_download_url(plat);
                download_file(&url, &chrome_dest).await.unwrap();
                extract_file(&chrome_dest, &current_dir).unwrap();
                if args.delete {
                    let _ = fs::remove_file(&chrome_dest);
                }
            }

            if dl_driver {
                let driver_name = format!("chromedriver-{}.zip", plat);
                let driver_dest = current_dir.join(&driver_name);
                println!("下载ChromeDriver for Testing");
                let url = UseChrome::ChromeDriver.generate_download_url(plat);
                download_file(&url, &driver_dest).await.unwrap();
                extract_file(&driver_dest, &current_dir).unwrap();
                if args.delete {
                    let _ = fs::remove_file(&driver_dest);
                }
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
            match process::check_running() {
                true => println!("服务已启动"),
                false => println!("服务未启动"),
            }
        }
        Commands::Start => {
            println!("启动服务...");
            process::start_auto_login();
        }

        Commands::Stop => {
            println!("停止服务...");
            process::stop_auto_login();
        }
    }
}