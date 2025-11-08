use anyhow::bail;
use flate2::write::GzDecoder;
use futures_util::StreamExt;
use reqwest::Client;
use std::fs::File;
use std::io;
use std::io::Write;
use std::path::PathBuf;
use tokio::fs;

const DOWNLOAD_URL: &'static str = "https://download.db-ip.com/free/dbip-city-lite-2025-11.csv.gz";

pub(crate) async fn start() -> anyhow::Result<PathBuf> {
    print!("📦  Downloading and compiling trie... ");
    io::stdout().flush()?;

    let directory = PathBuf::from("./data");
    if !directory.exists() {
        fs::create_dir_all(&directory).await?;
    }

    let file = directory.join("geo.csv");
    if file.exists() {
        // Already downloaded
        println!("Done");
        return Ok(file)
    }

    let client = Client::new();
    let response = client.get(DOWNLOAD_URL).send().await?;

    if !response.status().is_success() {
        bail!("HTTP request failed: {}", response.status());
    }

    let mut deflater = GzDecoder::new(File::create(&file)?);
    let mut stream = response.bytes_stream();

    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        deflater.write_all(&chunk)?;
    }

    deflater.flush()?;
    println!("Done");

    Ok(file)
}