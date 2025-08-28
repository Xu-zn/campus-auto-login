use anyhow::Result;
use futures_util::TryStreamExt;
use indicatif::{ProgressBar, ProgressStyle};
use std::{
    env::current_dir,
    fs::{File, read_to_string},
    io::Write,
    path::Path,
};

pub enum UseChrome {
    Chrome,
    ChromeDriver,
}

impl UseChrome {
    pub fn generate_download_url(self: &Self, platform: &str) -> String {
        let cur = current_dir().unwrap();
        let version_filepath = cur.join(".chrome-version");
        let version = read_to_string(version_filepath).unwrap_or("139.0.7258.138".to_string());

        match self {
            UseChrome::Chrome => {
                format!(
                    "https://registry.npmmirror.com/-/binary/chrome-for-testing/{version}/{platform}/chrome-{platform}.zip"
                )
            }

            UseChrome::ChromeDriver => {
                format!(
                    "https://registry.npmmirror.com/-/binary/chrome-for-testing/{version}/{platform}/chromedriver-{platform}.zip"
                )
            }
        }
    }
}

pub async fn download_file(url: &str, dest: &Path) -> Result<()> {
    let client = reqwest::Client::new();
    let resp = client.get(url).send().await?;
    resp.error_for_status_ref()?;

    let total_size = resp.content_length().unwrap_or(0);
    let pb = ProgressBar::new(total_size);
    pb.set_style(ProgressStyle::default_bar()
        .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta})")?
        .progress_chars("#>-"));

    let mut file = File::create(dest)?;
    let mut downloaded = 0u64;
    let mut stream = resp.bytes_stream();

    while let Some(chunk) = stream.try_next().await? {
        file.write_all(&chunk)?;
        let new = downloaded + chunk.len() as u64;
        downloaded = new;
        pb.set_position(new);
    }

    Ok(())
}
