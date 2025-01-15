use ipmb::label;

fn main () {
    // Join your bus 
    let options = ipmb::Options::new("campus.auto-login", label!("manager"), "");
    let (sender, _receiver) = ipmb::join::<String, bool>(options, None).expect("Join com.solar failed");


    // Create a message
    let selector = ipmb::Selector::unicast("serve");
    let message = ipmb::Message::new(selector, "exit".to_string());

    // Send the message
    sender.send(message).expect("当前可能未运行auto-login");
    println!("已退出auto-login");

    let _ = std::process::Command::new("cmd.exe").arg("/c").arg("pause").status();
}