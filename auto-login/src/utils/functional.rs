use tokio::time::{sleep, Duration};

pub async fn sleep_secs(secs: u64) {
    sleep(Duration::from_secs(secs)).await;
}

pub async fn sleep_millisecond(milli: u64) {
    sleep(Duration::from_millis(milli)).await;
}