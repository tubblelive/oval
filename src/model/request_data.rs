use serde::Deserialize;

#[derive(Deserialize)]
pub(crate) struct QueryAddress {
    pub address: String,
}