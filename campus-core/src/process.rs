use std::fs::OpenOptions;
use std::path::Path;
use fs4::fs_std::FileExt;

#[cfg(windows)]
use std::os::windows::process::CommandExt;

/// 检查 auto-login 守护进程是否正在运行（通过文件锁判断）
pub fn check_running() -> bool {
    let path = Path::new("lockfile");
    let file = match OpenOptions::new()
        .create(true)
        .read(true)
        .write(true)
        .open(path)
    {
        Ok(f) => f,
        Err(_) => return false,
    };
    !file.try_lock_exclusive().unwrap_or(true)
}

/// 启动 auto-login 守护进程
pub fn start_auto_login() {
    #[cfg(windows)]
    let result = std::process::Command::new("auto-login.exe")
        .creation_flags(0x08000000)
        .spawn();
    #[cfg(not(windows))]
    let result = std::process::Command::new("auto-login")
        .spawn();

    match result {
        Ok(_) => {}
        Err(e) => eprintln!("启动失败: {}", e),
    }
}

/// 向 auto-login 守护进程发送停止信号
#[cfg(windows)]
pub fn stop_auto_login() {
    use ipmb::label;
    let options = ipmb::Options::new("campus-login", label!("cli"), "");
    match ipmb::join::<String, String>(options, None) {
        Ok((sender, _)) => {
            let selector = ipmb::Selector::unicast("core");
            let message = ipmb::Message::new(selector, "exit".to_string());
            let _ = sender.send(message);
        }
        Err(e) => eprintln!("停止服务失败: {}", e),
    }
}

/// 通过 ipmb 向 auto-login 查询启动 Unix 时间戳（秒）
#[cfg(windows)]
pub fn query_uptime() -> Option<u64> {
    use ipmb::label;
    use std::time::Duration;
    let options = ipmb::Options::new("campus-login", label!("cli"), "");
    let (sender, mut receiver) = ipmb::join::<String, String>(options, None).ok()?;
    let selector = ipmb::Selector::unicast("core");
    let message = ipmb::Message::new(selector, "uptime".to_string());
    sender.send(message).ok()?;
    match receiver.recv(Some(Duration::from_secs(2))) {
        Ok(msg) => msg.payload.parse::<u64>().ok(),
        Err(_) => None,
    }
}

/// 非 Windows 平台上无法通过 ipmb 查询，始终返回 None
#[cfg(not(windows))]
pub fn query_uptime() -> Option<u64> {
    None
}

#[cfg(not(windows))]
pub fn stop_auto_login() {
    use std::io::Write;
    use std::os::unix::net::UnixStream;
    let socket_path = "/tmp/campus_login.sock";
    match UnixStream::connect(socket_path) {
        Ok(mut stream) => {
            let _ = stream.write_all(b"exit\n");
        }
        Err(e) => eprintln!("停止服务失败: {}", e),
    }
}
