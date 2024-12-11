use log::{debug, info, error};
use rumqttc::{Client, Event, Incoming, MqttOptions, QoS};
use serde::{Deserialize, Serialize};
use std::env;
use std::time::Duration;

#[derive(Debug, Serialize, Deserialize)]
struct OwntracksPayload {
    _type: String,
    _id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    acc: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    alt: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    batt: Option<i32>,
    bs: u8,
    cog: u8,
    conn: String,
    created_at: i32,
    lat: f64,
    lon: f64,
    m: u8,
    tid: String,
    tst: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    vac: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    vel: Option<i32>,
}

fn main() {
    env_logger::init();

    let mqtt_url = env::var("MQTT_BROKER_URL").unwrap_or_else(|_| "127.0.0.1".to_string());
    let mqtt_port: u16 = env::var("MQTT_BROKER_PORT")
        .unwrap_or_else(|_| "1883".to_string())
        .parse()
        .expect("MQTT_BROKER_PORT must be a valid number");
    let mqtt_username = env::var("MQTT_USERNAME").expect("MQTT_USERNAME must be set!");
    let mqtt_password = env::var("MQTT_PASSWORD").expect("MQTT_PASSWORD must be set!");
    let mqtt_topic = env::var("MQTT_TOPIC").expect("MQTT_TOPIC must be set!");
    let mqtt_keep_alive: u64 = env::var("MQTT_KEEP_ALIVE_DURATION")
        .unwrap_or_else(|_| "30".to_string())
        .parse()
        .expect("MQTT_KEEP_ALIVE_DURATION must be a valid number");

    let dawarich_api_key = env::var("DAWARICH_API_KEY").expect("DAWARICH_API_KEY must be set!");
    let dawarich_base_url =
        env::var("DAWARICH_BASE_URL").unwrap_or_else(|_| "127.0.0.1".to_string());
    let dawarich_port: u16 = env::var("DAWARICH_PORT")
        .unwrap_or_else(|_| "3000".to_string())
        .parse()
        .expect("DAWARICH_PORT must be a valid number");
    let dawarich_endpoint = format!(
        "http://{}:{}/api/v1/owntracks/points?api_key={}",
        dawarich_base_url, dawarich_port, dawarich_api_key
    );

    info!("Sending data to Dawarich at {}:{}", dawarich_base_url, dawarich_port);

    let client = "mqtt2dawarich";


    let mut mqttoptions = MqttOptions::new(client, mqtt_url.clone(), mqtt_port.clone());
    mqttoptions.set_keep_alive(Duration::from_secs(mqtt_keep_alive));
    mqttoptions.set_credentials(mqtt_username, mqtt_password);

    let (client, mut connection) = Client::new(mqttoptions.clone(), 10);

    client.subscribe(mqtt_topic.clone(), QoS::AtMostOnce).unwrap();
    info!("Listening to MQTT broker on {}:{} for topic {}", mqtt_url, mqtt_port, mqtt_topic);
    let d_client = reqwest::blocking::Client::new();

    for notification in connection.iter() {
        match notification {
            Ok(notif) => match notif {
                Event::Incoming(Incoming::Publish(package)) => {
                    let op: OwntracksPayload = serde_json::from_slice(&package.payload).unwrap();

                    let response = d_client.post(&dawarich_endpoint).json(&op).send();

                    match response {
                        Ok(resp) => debug!("Response: {resp:?}"),
                        Err(err) => error!("Request failed with error: {err:?}"),
                    }
                }
                _ => debug!("Ignoring non-payload message"),
            },
            Err(err) => error!("{err:?}"),
        }
    }
}
