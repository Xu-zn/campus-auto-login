// #![cfg(not(debug_assertions))]
#![windows_subsystem = "windows"]

use std::{env::current_dir, fs::OpenOptions, path::Path};
use tklog::{info, error, Format, LEVEL, LOG, MODE};
use tokio_util::sync::CancellationToken;
use fs4::fs_std::FileExt;

use auto_login::{
    task::{task_detection, task_stop},
};
use campus_core::config::ConfigFile;
use campus_core::elements::Elements;
use campus_core::errors::CampusError;
use auto_login::{CONFIG, ELEMENTS};

#[tokio::main]
async fn main() {
    let path = Path::new("lockfile");

    // 打开文件或创建它（如果不存在），但不截断已有内容
    let file = OpenOptions::new()
        .create(true)
        .read(true)
        .write(true)
        .open(&path)
        .expect("创建或打开加锁的文件失败");

    let lock = file.try_lock_exclusive().unwrap();

    if !lock {
        error!("已有相同实例在运行");
        return;
    }

    // 设置日志格式
    LOG.set_console(false)
        .set_level(LEVEL::Info)
        .set_format(Format::LevelFlag | Format::Time)
        .set_cutmode_by_time("neco.log", MODE::DAY, 7, false)
        .set_formatter("{level} {time}: {message}\n");

    if let Err(e) = load_config() {
        error!("配置加载失败：", e);
        return;
    }
    if let Err(e) = load_elements() {
        error!("页面元素配置加载失败：", e);
        return;
    }

    // 创建 CancellationToken 用于任务取消
    let cancel_token = CancellationToken::new();
    let cancel_token_for_network_check = cancel_token.clone();

    info!("AutoLogin开始运行");

    let net_check = task_detection(cancel_token_for_network_check);

    // 启动一个异步任务来监听停止信号
    let stop_signal_task = task_stop();

    stop_signal_task.await.ok();
    cancel_token.cancel();

    // 等待网络检查任务完成（如果存在）
    if let Err(e) = net_check.await {
        error!("网络检查任务异常退出: ", e);
    }
    info!("AutoLogin已退出");
}


fn load_config() -> Result<(), CampusError> {
    let config_path = current_dir().unwrap().join("config.toml");
    info!("配置文件路径: ", config_path.display());
    let conf = ConfigFile::load_config(&config_path)?;
    let _ = CONFIG.set(conf);
    Ok(())
}


fn load_elements() -> Result<(), CampusError> {
    let elements_path = current_dir().unwrap().join("elements.toml");
    let elements = Elements::load_file(&elements_path)?;
    let _ = ELEMENTS.set(elements);
    Ok(())
}