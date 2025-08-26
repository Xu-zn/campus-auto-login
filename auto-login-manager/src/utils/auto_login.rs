use ipmb::label;
use tokio::process::Command;

pub fn stop_auto_login() {
    let options = ipmb::Options::new("campus.auto-login", label!("manager"), "");
    let (sender, _receiver) = ipmb::join::<String, bool>(options, None).expect("Join com.solar failed");
    // Create a message
    let selector = ipmb::Selector::unicast("serve");
    let message = ipmb::Message::new(selector, "exit".to_string());

    // Send the message
    let res = sender.send(message);
    if res.is_err() {

    }
}

pub fn start_auto_login() {
    #[cfg(windows)]
    let _cmd = Command::new("auto-login.exe")
        .creation_flags(0x08000000)
        .spawn();
    #[cfg(not(windows))]
    let _cmd = Command::new("auto-login")
        .spawn();
 }