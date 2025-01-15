// #![cfg(not(debug_assertions))]
#![windows_subsystem = "windows"]

use std::{
    env::current_dir, time::Duration,
};




use tklog::{error, info, Format, LEVEL, LOG, MODE};

use tokio_util::sync::CancellationToken;

use tokio::time::sleep;

use auto_login:: {
    sleep_secs, toast, init_config,
    net::check::check_lan,
    task::{
        task_netcheck,
        task_stop
    }
};

use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;




#[tokio::main]
async fn main() {
    // 创建一个全局唯一的互斥锁名称
    let mutex_name: Vec<u16> = OsStr::new("CampusAutoLogin")
        .encode_wide()
        .chain(Some(0).into_iter())
        .collect();

    // 尝试获取互斥锁
    unsafe {
        let mutex = windows_sys::Win32::System::Threading::CreateMutexW(
            std::ptr::null_mut(),
            0,
            mutex_name.as_ptr(),
        );

        if mutex.is_null() || windows_sys::Win32::Foundation::GetLastError() == windows_sys::Win32::Foundation::ERROR_ALREADY_EXISTS {
            toast("已运行相同程序");
            return;
        } else {
            // 设置日志格式
            LOG.set_console(false)
            .set_level(LEVEL::Info)
            .set_format(Format::LevelFlag | Format::Time)
            .set_cutmode_by_time("neco.log", MODE::DAY, 7, false)
            .set_formatter("{level} {time}: {message}\n");

            // 获取当前文件所在目录
            let curdir = match current_dir() {
                Ok(curdir) => curdir,
                Err(_) => {
                    let message = "读取当前目录失败";
                    error!(&message);
                    toast(message);
                    return;
                }
            };

            let config = match init_config(curdir) {
                Ok(t) => t,
                Err(_) => {
                    return;
                }
            };

            let mut wait_time: u16 = 0;
            let mut lan_is_connected = true;

            while let Err(_) = check_lan() {
                if wait_time >= 60 {
                    lan_is_connected = false;
                    break;
                }

                info!("等待局域网连接");
                sleep_secs(1).await;
                wait_time += 1;
            };

            if !lan_is_connected {
                error!("局域网连接超时，程序已退出");
            };

            info!("局域网已连接");

            // 创建 CancellationToken 用于任务取消
            let cancel_token = CancellationToken::new();
            let cancel_token_for_network_check = cancel_token.clone();

            info!("AutoLogin开始运行");
            toast("AutoLogin开始运行");

            let netcheck = task_netcheck(config.clone(), cancel_token_for_network_check);

            

            // 启动一个异步任务来监听停止信号
            let mut stop_signal_task = task_stop();


            // 主线程中的 loop，监听取消信号
            loop {
                tokio::select! {
                    _ = sleep(Duration::from_secs(10)) => {
                        // 主线程的正常逻辑
                    },
                    _ = &mut stop_signal_task => {
                        info!("接收到停止通知");
                        cancel_token.cancel();
                        netcheck.await.expect("等待netcheck任务异常");
                        
                        // 收到取消信号，退出 loop                
                        break;
                    }

                }
            };

            info!("AutoLogin已退出");
            toast("AutoLogin已退出");
            // 释放互斥锁
            windows_sys::Win32::System::Threading::ReleaseMutex(mutex);
            windows_sys::Win32::Foundation::CloseHandle(mutex);
        }
    }

   
}




