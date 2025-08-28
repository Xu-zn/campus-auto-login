use clap::Parser;
use std::{env::current_dir, fs};
use manager_cli::utils::{
    args::{Cli, Commands},
    download::{UseChrome, download_file},
    extract::extract_file,
    platform::detect_platform,
};

use auto_login_common::{config::ConfigFile};

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Download(args) => {
            // 检测平台
            let platform = detect_platform();
            
            let plat = args.platform.as_ref().unwrap_or(&platform);
            // 下载后的文件名
            let chrome_name = format!("chrome-{}.zip", &plat);
            let current_dir = std::env::current_dir().unwrap();
            let chrome_dest = current_dir.join(chrome_name);
            // 下载 Chrome 浏览器
            let chrome_url = UseChrome::Chrome.generate_download_url(&plat);
            println!("下载Chrome for Test");
            download_file(&chrome_url, &chrome_dest).await.unwrap();
            // 解压缩
            extract_file(&chrome_dest, &current_dir).unwrap();

            let driver_name = format!("chromedriver-{}.zip", &plat);
            // let current_dir = std::env::current_dir().unwrap();
            let driver_dest = current_dir.join(driver_name);
            // 下载 Chrome 浏览器
            let chrome_url = UseChrome::ChromeDriver.generate_download_url(&plat);
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
            match check_running() {
                true => println!("服务已启动"),
                false => println!("服务未启动"),
            }
        }
        Commands::Start => {
            println!("启动服务...");
            start_auto_login();
        }

        Commands::Stop => {
            println!("停止服务...");
            stop_auto_login().await;
        }
    }
}

#[cfg(not(windows))]
pub async fn stop_auto_login() {
    use tokio::net::UnixStream;
    use tokio::io::AsyncWriteExt;
    let socket_path = "/tmp/campus_login.sock";
    let mut stream = UnixStream::connect(socket_path).await.unwrap();
    // 直接发送 exit
    stream.write_all(b"exit\n").await.unwrap();
}

#[cfg(windows)]
pub async fn stop_auto_login() {
    use ipmb::label;
    let options = ipmb::Options::new("campus-login", label!("cli"), "");
    let (sender, _receiver) = ipmb::join::<String, String>(options, None).unwrap();

    let selector = ipmb::Selector::unicast("core");
    let message = ipmb::Message::new(selector, "exit".to_string());

    // Send the message
    sender.send(message).unwrap();
}

use tokio::process::Command;
pub fn start_auto_login() {
    #[cfg(windows)]
    let _cmd = Command::new("auto-login.exe")
        .creation_flags(0x08000000)
        .spawn();
    #[cfg(not(windows))]
    let _cmd = Command::new("auto-login")
        .spawn();

    if _cmd.is_err() {
        println!("启动失败{}", _cmd.err().unwrap().to_string());
    }

 }

use std::path::Path;
use std::fs::OpenOptions;
use fs4::fs_std::FileExt;

fn check_running() -> bool {
    let path = Path::new("lockfile");
    // 打开文件或创建它（如果不存在），但不截断已有内容
    let file = OpenOptions::new()
        .create(true)
        .read(true)
        .write(true)
        .open(&path)
        .expect("创建或打开加锁的文件失败");

    !file.try_lock_exclusive().unwrap()
}