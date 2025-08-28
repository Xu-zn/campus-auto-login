pub fn stop_auto_login() {
    #[cfg(windows)]
    let _cmd = tokio::process::Command::new("auto-login.exe").arg("stop")
        .creation_flags(0x08000000)
        .spawn();
    #[cfg(not(windows))]
    let _cmd = std::process::Command::new("auto-login").arg("stop").output().expect("应用停止失败");
}

pub fn start_auto_login() {
    #[cfg(windows)]
    let _cmd = tokio::process::Command::new("auto-login.exe")
        .creation_flags(0x08000000)
        .spawn();
    #[cfg(not(windows))]
    let _cmd = std::process::Command::new("auto-login")
        .spawn();
 }