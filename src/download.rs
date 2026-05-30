use anyhow::bail;
use chrono::{Datelike, Duration, Utc};
use flate2::write::GzDecoder;
use futures_util::StreamExt;
use reqwest::{Client, Response};
use std::fs::File;
use std::io;
use std::io::Write;
use std::path::PathBuf;
use tokio::fs;

const MAX_ATTEMPTS: i32 = 3;
const DOWNLOAD_URL: &'static str = "https://download.db-ip.com/free/dbip-city-lite-{year}-{month}.csv.gz";

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
        println!("Done (using local)");
        return Ok(file)
    }

    let client = Client::new();
    let mut time = Utc::now();
    let mut response: Option<Response> = None;

    for _ in 0..MAX_ATTEMPTS {
        let url = DOWNLOAD_URL
            .replace("{year}", &time.year().to_string())
            .replace("{month}", &format!("{:02}", time.month()));

        let res = client.get(&url).send().await?;
        let success = res.status().is_success();

        if !success {
            time -= Duration::days(30);
            continue;
        }

        response = Some(res);
        break;
    }

    let response = match response {
        Some(value) => value,
        None => bail!("HTTP request failed, could not download database"),
    };

    let mut deflater = GzDecoder::new(File::create(&file)?);
    let mut stream = response.bytes_stream();

    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        deflater.write_all(&chunk)?;
    }

    deflater.flush()?;
    println!("Done (version {}-{})", time.year(), time.month());

    Ok(file)
}
