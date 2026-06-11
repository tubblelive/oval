extern crate core;

use crate::model::fail::Fail;
use crate::model::geo_data::GeoData;
use crate::model::request_data::QueryAddress;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Result};
use axum::routing::{get, post};
use axum::{Json, Router};
use ipnet::IpNet;
use iptrie::IpLCTrieMap;
use std::net::IpAddr;
use std::sync::Arc;
use tokio::net::TcpListener;
use clap::Parser;

mod download;
mod model;
mod reader;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// The port to use when running oval
    #[arg(short, long, default_value_t = 8080, env = "PORT")]
    port: i32,
}

#[derive(Clone)]
pub(crate) struct AppState {
    pub tries: Arc<Vec<IpLCTrieMap<Arc<GeoData>>>>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let file = download::start().await?;
    let trie = reader::parse_csv(file).await?;

    let state = AppState {
        tries: Arc::new(trie),
    };

    tracing_subscriber::fmt::init();

    let app = Router::new()
        .route("/healthz", get(health))
        .route("/query", post(query))
        .with_state(state);

    let listener = TcpListener::bind(format!("[::]:{}", args.port)).await?;
    println!("Listening on port {}", args.port);

    axum::serve(listener, app).await?;
    Ok(())
}

async fn health() -> &'static str {
    "OK"
}

async fn query(
    State(state): State<AppState>,
    Json(payload): Json<QueryAddress>,
) -> Result<impl IntoResponse, Fail> {
    let address = payload.address;

    let empty_addr = "0.0.0.0".parse::<IpAddr>().unwrap();
    let net: IpNet = address
        .parse()
        .map_err(|error| Fail::new(format!("Invalid address provided! {error}")))?;

    let data = state
        .tries
        .iter()
        .map(|it| it.lookup(&net).1)
        .filter(|it| !it.start.eq(&empty_addr))
        .nth(0);

    match data {
        Some(data) => Ok((StatusCode::OK, Json::from(data.as_ref().clone()))),
        None => Err(Fail::new("No data was found for the provided address")),
    }
}
