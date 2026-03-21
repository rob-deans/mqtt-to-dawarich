use serde::{Deserialize, Serialize};
use std::env;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DawarichConfig {
    pub url: String,
    pub api_key: String,

    pub port: u16,
    pub endpoint: String,
}

impl DawarichConfig {
    pub fn from_env() -> Self {
        let dawarich_api_key = env::var("DAWARICH_API_KEY").expect("DAWARICH_API_KEY must be set!");
        let dawarich_base_url =
            env::var("DAWARICH_BASE_URL").unwrap_or_else(|_| "127.0.0.1".to_string());
        let dawarich_port: u16 = env::var("DAWARICH_PORT")
            .unwrap_or_else(|_| "3000".to_string())
            .parse()
            .expect("DAWARICH_PORT must be a valid number");
        let dawarich_endpoint = format!(
            "http://{}:{}/api/v1/owntracks/points",
            dawarich_base_url, dawarich_port
        );

        Self {
            url: dawarich_base_url,
            api_key: dawarich_api_key,
            port: dawarich_port,
            endpoint: dawarich_endpoint,
        }
    }
}
