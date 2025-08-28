use std::fs::File;
use std::io;
use std::path::Path;

pub fn extract_file<P: AsRef<Path>>(zip_path: P, extract_to: P) -> io::Result<()> {
    // 1. 打开 ZIP 文件
    let file = File::open(zip_path)?;
    
    // 2. 创建 ZipArchive 读取器
    let mut archive = zip::ZipArchive::new(file)?;
    
    // 3. 遍历 ZIP 中的每一个文件/目录
    for i in 0..archive.len() {
        let mut file_in_zip = archive.by_index(i)?;
        
        // 获取在 ZIP 中的文件名，并进行安全处理（防止路径穿越攻击）
        let outpath = Path::new(extract_to.as_ref()).join(file_in_zip.mangled_name());
        
        // 4. 根据条目类型处理
        if file_in_zip.name().ends_with('/') {
            // 是一个目录
            println!("创建目录: {:?}", outpath);
            std::fs::create_dir_all(&outpath)?;
        } else {
            // 是一个文件
            println!("解压文件: {:?}", outpath);
            
            // 确保父目录存在
            if let Some(parent) = outpath.parent() {
                std::fs::create_dir_all(parent)?;
            }
            
            // 创建目标文件并写入内容
            let mut output_file = File::create(&outpath)?;
            io::copy(&mut file_in_zip, &mut output_file)?;
        }
        
        // 5. (可选) 设置文件权限 (Unix-like 系统)
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            if let Some(mode) = file_in_zip.unix_mode() {
                std::fs::set_permissions(&outpath, std::fs::Permissions::from_mode(mode))?;
            }
        }
    }
    
    Ok(())
}
