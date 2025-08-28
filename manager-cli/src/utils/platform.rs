use std::env;
pub fn detect_platform() -> String {
    let os = env::consts::OS;
    let arch = env::consts::ARCH;

    match (os, arch) {
        ("windows", "x86") => String::from("win32"),
        ("windows", "x86_64") => String::from("win64"),
        ("linux", "x86_64") => String::from("linux64"),
        _ => panic!("Unsupported platform: {}-{}", os, arch),
    }
}
