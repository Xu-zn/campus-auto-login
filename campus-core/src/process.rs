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
        Ok((sender, _receiver)) => {
            let selector = ipmb::Selector::unicast("core");
            let message = ipmb::Message::new(selector, "exit".to_string());
            let _ = sender.send(message);
        }
        Err(e) => eprintln!("停止服务失败: {}", e),
    }
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
