use reqwest::Client;

#[tokio::main]
async fn main() {
    let client = Client::new();
    let resp = client.get("https://captive.apple.com/hotspot-detect.html").send().await.unwrap();

    println!("{}", resp.text().await.unwrap());
}
