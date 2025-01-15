use reqwest::Client;
use crate::config::{ConnectionState, ResultAsync};

// use windows::Win32::Networking::WinInet::{
//     InternetGetConnectedState, INTERNET_CONNECTION
// };


// /// 检测当前是否插入网线或接入无线网
// pub fn check_lan() -> Result<(), windows::core::Error> {
//     let mut flag = INTERNET_CONNECTION(0);
//     let connected = unsafe { InternetGetConnectedState(&mut flag, 0) };
//     connected
// }

use windows_sys::Win32::Networking::WinInet::InternetGetConnectedState;

pub fn check_lan() -> Result<(), u8> {
    let mut flag = 0;
    let connected = unsafe { 
        InternetGetConnectedState(&mut flag, 0) };

    if connected != 0 {
        Ok(())
    } else {
        Err(0)
    }
}


/// 检测当前网络是否能访问外网
pub async fn check_connection(connection: &Vec<ConnectionState>, client: &Client) -> Result<(), ResultAsync> {
    
    let mut index = 0;
    let max_length = connection.len();

    let request_success = loop {
        if index >= max_length {
            break false;
        }
        let conn = connection.get(index).unwrap();
        let res = client.get(&conn.url).send().await;
        if res.is_ok() {
            let res = res.unwrap();
            let res_status = res.status();
            let res_text = res.text().await?;
            if res_status.is_success() && res_text.eq(&conn.value) {
                break true;
            }
        }
        index += 1;
    };
    if request_success {
        Ok(())
    }
    else {
        Err("网络未连接".into())
    }

    
}
