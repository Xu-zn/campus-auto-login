// #![cfg(not(debug_assertions))]
#![windows_subsystem = "windows"]

use std::{env::current_dir, fs::OpenOptions, path::Path};
use std::time::{SystemTime, UNIX_EPOCH};
use tklog::{info, error, Format, LEVEL, LOG, MODE};
use tokio_util::sync::CancellationToken;
use fs4::fs_std::FileExt;

use auto_login::{
    task::{task_detection, task_stop},
};
use campus_core::config::ConfigFile;
use campus_core::elements::Elements;
use campus_core::errors::CampusError;
use auto_login::{CONFIG, ELEMENTS, STARTUP_TIMESTAMP};

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

    // 记录启动时间戳，供 GUI 通过 ipmb 查询
    let _ = STARTUP_TIMESTAMP.set(
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs(),
    );

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

    // ── 校验配置值 ──
    validate_config(&conf)?;

    let _ = CONFIG.set(conf);
    Ok(())
}

/// 校验配置各字段不能为空
fn validate_config(conf: &ConfigFile) -> Result<(), CampusError> {
    let mut errors: Vec<&str> = Vec::new();

    if conf.login.info.username.trim().is_empty() { errors.push("用户名不能为空"); }
    if conf.login.info.password.trim().is_empty() { errors.push("密码不能为空"); }
    if conf.login.info.service.trim().is_empty() { errors.push("服务商不能为空"); }
    if conf.login.config.eportal.trim().is_empty() { errors.push("门户地址不能为空"); }
    if conf.login.config.timout == 0 { errors.push("超时不能为0"); }

    if conf.query.interval == 0 { errors.push("检测间隔不能为0"); }
    if conf.query.connect.is_empty() { errors.push("连接检测列表不能为空"); }
    let mut has_empty_url = false;
    let mut has_empty_val = false;
    for conn in &conf.query.connect {
        if conn.url.trim().is_empty() { has_empty_url = true; }
        if conn.val.trim().is_empty() { has_empty_val = true; }
    }
    if has_empty_url { errors.push("连接条目URL不能为空"); }
    if has_empty_val { errors.push("连接条目值不能为空"); }

    match &conf.driver.chrome_config {
        Some(chrome) => {
            if chrome.port == 0 { errors.push("驱动端口不能为0"); }
            if chrome.driver_path.trim().is_empty() { errors.push("Driver路径不能为空"); }
            if chrome.browser_path.trim().is_empty() { errors.push("浏览器路径不能为空"); }
        }
        None => errors.push("Chrome驱动配置缺失"),
    }

    if !errors.is_empty() {
        return Err(CampusError::ConfigValidation(errors.join("；")));
    }
    Ok(())
}


fn load_elements() -> Result<(), CampusError> {
    let elements_path = current_dir().unwrap().join("elements.toml");
    let elements = Elements::load_file(&elements_path)?;
    let _ = ELEMENTS.set(elements);
    Ok(())
}