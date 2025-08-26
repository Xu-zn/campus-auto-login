use std::env;

#[derive(Debug, PartialEq)]
pub enum Platform {
    Win32,
    Win64,
    Linux64,
}

impl Platform {
    pub fn to_string(&self) -> &str {
        match self {
            Platform::Win32 => "win32",
            Platform::Win64 => "win64",
            Platform::Linux64 => "linux64",
        }
    }
}

pub fn detect_platform() -> Platform {
    let os = env::consts::OS;
    let arch = env::consts::ARCH;

    match (os, arch) {
        ("windows", "x86") => Platform::Win32,
        ("windows", "x86_64") => Platform::Win64,
        ("linux", "x86_64") => Platform::Linux64,
        _ => panic!("Unsupported platform: {}-{}", os, arch),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_platform() {
        // This test is platform-dependent and may need adjustment
        let platform = detect_platform();
        assert!(matches!(
            platform,
            Platform::Win32 | Platform::Win64 | Platform::Linux64
        ));
    }
}