use log::{debug, error, info};
use std::{cmp, env};

use crate::models::{owntracks::OwntracksPayload, persistent_queue::PersistentQueue};

#[derive(Debug, Clone)]
enum ApiStatus {
    Healthy,
    Degraded,
}

#[derive(Debug, Clone)]
pub struct Dawarich {
    client: reqwest::blocking::Client,
    api_key: String,
    endpoint: String,
    status: ApiStatus,
    concurrent_successes: u16,
    queue: PersistentQueue,
}

impl Dawarich {
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
        let filepath = env::var("EVENT_LOG").unwrap_or_else(|_| "checkpoint.ndjson".to_string());

        info!(
            "Sending data to Dawarich at {}:{}",
            dawarich_endpoint, dawarich_port
        );

        Self {
            client: reqwest::blocking::Client::new(),
            api_key: dawarich_api_key,
            endpoint: dawarich_endpoint,
            status: ApiStatus::Healthy,
            concurrent_successes: 0,
            queue: PersistentQueue::new(filepath),
        }
    }

    pub fn push(&mut self, payload: OwntracksPayload) {
        let response = self
            .client
            .post(&self.endpoint)
            .json(&payload)
            .bearer_auth(&self.api_key)
            .send();

        match response {
            Ok(resp) => {
                debug!("Response: {resp:?}");
                self.concurrent_successes = cmp::min(self.concurrent_successes + 1, u16::MAX - 1);
                debug_assert!(self.concurrent_successes <= 3);

                if self.concurrent_successes >= 3 {
                    self.status = ApiStatus::Healthy;
                }
            }
            Err(err) => {
                error!("Request failed with error: {err:?}");
                self.concurrent_successes = 0;
                debug!("Setting status to degraded");
                self.status = ApiStatus::Degraded;

                self.queue.push(payload);
            }
        }
    }

    /*
     * Push onto payload queue then if the API connection is not degraded, flush
     */
    pub fn write(&mut self, payload: OwntracksPayload) {
        self.queue.push(payload);

        while let Some(data) = self.queue.pop() {
            self.push(data);

            match self.status {
                ApiStatus::Degraded => {
                    // results in at least this + 2 new requests coming though successfully before flushing
                    break;
                }
                ApiStatus::Healthy => {}
            }
        }
    }
}
