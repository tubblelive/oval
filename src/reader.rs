use crate::model::geo_data::GeoData;
use anyhow::bail;
use ipnet::{Ipv4Subnets, Ipv6Subnets};
use iptrie::{IpLCTrieMap, IpRTrieMap, Ipv4Prefix, Ipv6Prefix};
use std::net::IpAddr;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;

pub(crate) async fn parse_csv(file: PathBuf) -> anyhow::Result<IpLCTrieMap<Arc<GeoData>>> {
    let mut reader = csv::ReaderBuilder::new()
        .has_headers(false)
        .double_quote(true)
        .delimiter(b',')
        .from_path(file)?;

    let started = Instant::now();
    let mut trie: IpRTrieMap<Arc<GeoData>> = IpRTrieMap::new();

    for result in reader.deserialize::<GeoData>() {
        let result = Arc::new(result?);

        match (result.start, result.end) {
            (IpAddr::V4(start), IpAddr::V4(end)) => {
                for net in Ipv4Subnets::new(start, end, 0) {
                    trie.ipv4.insert(Ipv4Prefix::from(net), result.clone());
                }
            },
            (IpAddr::V6(start), IpAddr::V6(end)) => {
                for net in Ipv6Subnets::new(start, end, 0) {
                    trie.ipv6.insert(Ipv6Prefix::from(net), result.clone());
                }
            }
            _ => bail!("Found IP version mismatch between {} and {}", result.start, result.end)
        }
    }

    let duration = Instant::now().duration_since(started);
    println!("Created trie in {:.2?}", duration);

    Ok(trie.compress())
}