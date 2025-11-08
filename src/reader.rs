use std::io;
use std::io::Write;
use crate::model::geo_data::GeoData;
use anyhow::bail;
use futures_util::future::join_all;
use ipnet::{Ipv4Subnets, Ipv6Subnets};
use iptrie::{IpLCTrieMap, IpRTrieMap, Ipv4Prefix, Ipv6Prefix};
use std::net::IpAddr;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;
use tokio::task::{self, JoinHandle};

pub(crate) async fn parse_csv(file: PathBuf) -> anyhow::Result<Vec<IpLCTrieMap<Arc<GeoData>>>> {
    print!("📋  Parsing geo-ip database... ");
    io::stdout().flush()?;

    let started = Instant::now();
    let mut reader = csv::ReaderBuilder::new()
        .has_headers(false)
        .double_quote(true)
        .delimiter(b',')
        .from_path(file)?;

    let mut data: Vec<Arc<GeoData>> = Vec::with_capacity(10_000_000);

    for result in reader.deserialize::<GeoData>() {
        data.push(Arc::new(result?));
    }

    let duration = Instant::now().duration_since(started);
    println!("Done in {:.2?}", duration);
    print!("🌲  Creating tries... ");
    io::stdout().flush()?;

    let started = Instant::now();
    let jobs = data
        .chunks(200_000)
        .map(|chunk| create_job(chunk.to_vec()))
        .collect::<Vec<_>>();

    let results = join_all(jobs).await;
    let mut subtries: Vec<IpLCTrieMap<Arc<GeoData>>> = Vec::with_capacity(data.len() / 200_000);

    for subtrie in results {
        let subtrie = subtrie??;
        subtries.push(subtrie)
    }

    let duration = Instant::now().duration_since(started);
    println!("Done in {:.2?}", duration);

    Ok(subtries)
}

fn create_job(chunk: Vec<Arc<GeoData>>) -> JoinHandle<anyhow::Result<IpLCTrieMap<Arc<GeoData>>>> {
    task::spawn(async move {
        let mut subtrie = IpRTrieMap::new();

        for result in chunk {
            match (result.start, result.end) {
                (IpAddr::V4(start), IpAddr::V4(end)) => {
                    for net in Ipv4Subnets::new(start, end, 0) {
                        subtrie.ipv4.insert(Ipv4Prefix::from(net), result.clone());
                    }
                }
                (IpAddr::V6(start), IpAddr::V6(end)) => {
                    for net in Ipv6Subnets::new(start, end, 0) {
                        subtrie.ipv6.insert(Ipv6Prefix::from(net), result.clone());
                    }
                }
                _ => bail!(
                    "Found IP version mismatch between {} and {}",
                    result.start,
                    result.end
                ),
            }
        }

        Ok(subtrie.compress())
    })
}
