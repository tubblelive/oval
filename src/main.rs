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
use std::env;
use std::sync::Arc;
use tokio::net::TcpListener;

mod download;
mod reader;
mod model;

#[derive(Clone)]
pub(crate) struct AppState {
    pub trie: Arc<IpLCTrieMap<Arc<GeoData>>>
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("Downloading and compiling trie...");

    let file = download::start().await?;
    let trie = reader::parse_csv(file).await?;

    let state = AppState {
        trie: Arc::new(trie)
    };

    tracing_subscriber::fmt::init();

    let app = Router::new()
        .route("/healthz", get(health))
        .route("/query", post(query))
        .with_state(state);

    let port = env::var("PORT").unwrap_or("8080".to_owned());

    let listener = TcpListener::bind(format!("0.0.0.0:{}", port)).await?;
    println!("Listening on port {}", port);

    axum::serve(listener, app).await?;
    Ok(())
}

async fn health() -> &'static str {
    "OK"
}

async fn query(
    State(state): State<AppState>,
    Json(payload): Json<QueryAddress>
) -> Result<impl IntoResponse, Fail> {
    let address = payload.address;

    let net: IpNet = address.parse()
        .map_err(|error| Fail::new(format!("Invalid address provided! {error}")))?;

    let (_, data) = state.trie.lookup(&net);
    let data = data.as_ref().clone();

    Ok((StatusCode::OK, Json::from(data)))
}