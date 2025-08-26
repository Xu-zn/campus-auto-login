use std::env::current_dir;
use auto_login_common::{ config::ConfigFile, detect::detect_network_status };

#[tokio::main]
async fn main() {
    let client = reqwest::Client::new();
    let p = current_dir().unwrap();
    ConfigFile::create_default_config(&p).unwrap();
    let p = p.join("config.toml");
    let config = ConfigFile::load_config(&p).unwrap();
    let status = detect_network_status(&config.query, &client).await.unwrap();
    println!("{}", status.display());
}