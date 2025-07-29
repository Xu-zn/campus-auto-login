#![windows_subsystem = "windows"]

slint::include_modules!();

use auto_login_manager::utils::auto_login::{stop_auto_login, start_auto_login};
use std::fs::OpenOptions;
use std::path::Path;
use fs4::fs_std::FileExt;

fn main () {
    let manager_window = ManagerWindow::new().unwrap();
    let running_status = manager_window.as_weak();

    manager_window.on_check_running_status( move |cur| {
        let path = Path::new("lockfile");
        // 打开文件或创建它（如果不存在），但不截断已有内容
        let file = OpenOptions::new()
            .create(true)
            .read(true)
            .write(true)
            .open(&path)
            .expect("创建或打开加锁的文件失败");

        let lock = file.try_lock_exclusive().unwrap();
        if !lock && cur != 1 {
            running_status.upgrade().as_ref().unwrap().set_running_status(1);
        } else if lock && cur != 2{
            running_status.upgrade().as_ref().unwrap().set_running_status(2);
        } else {}
    });
    manager_window.on_update_running_status(|cur: i32| {
        return if cur == 1 {
            stop_auto_login();
        } else {
            start_auto_login();
        }
    });
    
    manager_window.run().unwrap();
}