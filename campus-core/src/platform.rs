/// 检测当前平台标识符（用于 Chrome 下载等场景）
pub fn detect_platform() -> String {
    let os = std::env::consts::OS;
    let arch = std::env::consts::ARCH;

    match (os, arch) {
        ("windows", "x86") => String::from("win32"),
        ("windows", "x86_64") => "win64".into(),
        ("windows", "aarch64") => "win64".into(),
        ("linux", "x86_64") => "linux64".into(),
        _ => panic!("Unsupported platform: {}-{}", os, arch),
    }
}

/// 获取当前平台的可执行文件扩展名
pub fn exe_suffix() -> &'static str {
    if cfg!(windows) { ".exe" } else { "" }
}
