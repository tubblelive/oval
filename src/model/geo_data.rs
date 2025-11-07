use serde::{Deserialize, Serialize};
use std::net::IpAddr;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GeoData {
    pub start: IpAddr,
    pub end: IpAddr,

    pub continent: String,
    pub country: String,
    pub province: String,
    pub city: String,

    pub latitude: f64,
    pub longitude: f64,
}

impl Default for GeoData {
    fn default() -> Self {
        GeoData {
            start: "0.0.0.0".parse().unwrap(),
            end: "0.0.0.0".parse().unwrap(),
            continent: "".to_string(),
            country: "".to_string(),
            province: "".to_string(),
            city: "".to_string(),
            latitude: 0f64,
            longitude: 0f64,
        }
    }
}