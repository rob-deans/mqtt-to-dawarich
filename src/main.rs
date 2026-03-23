use log::{debug, error, info};
use models::{dawarich, owntracks};
use rumqttc::{Client, Event, Incoming, MqttOptions, QoS};
use std::env;
use std::time::Duration;

pub mod models;

fn main() {
    env_logger::init();

    let mut dawarich = dawarich::Dawarich::from_env();

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

    let client = "mqtt2dawarich-local";

    let mut mqttoptions = MqttOptions::new(client, mqtt_url.clone(), mqtt_port);
    mqttoptions.set_keep_alive(Duration::from_secs(mqtt_keep_alive));
    mqttoptions.set_credentials(mqtt_username, mqtt_password);

    let (client, mut connection) = Client::new(mqttoptions, 10);

    for notification in connection.iter() {
        match notification {
            Ok(Event::Incoming(Incoming::ConnAck(connack))) => {
                debug!(
                    "ConnAck received. Attempting to subscribe to topic. {:?}",
                    connack
                );
                info!(
                    "Listening to MQTT broker on {}:{} for topic {}",
                    mqtt_url, mqtt_port, mqtt_topic
                );
                if let Err(err) = client.subscribe(&mqtt_topic, QoS::AtMostOnce) {
                    error!("failed to resubscribe {:?}", err);
                }
            }
            Ok(notif) => match notif {
                Event::Incoming(Incoming::Publish(package)) => {
                    match serde_json::from_slice::<owntracks::OwntracksPayload>(&package.payload) {
                        Ok(data) => {
                            if data._type != "location" {
                                debug!("ignoring non-location payload type {:?}", data._type);
                                continue;
                            }

                            dawarich.write(data);
                        }

                        Err(err) => {
                            error!("Something went wrong with deserialising the payload");
                            error!("Error: {err}");
                        }
                    }
                }
                p => debug!("Ignoring non-payload message {:?}", p),
            },
            Err(err) => {
                error!("{err:?}");
                std::thread::sleep(Duration::from_secs(3));
            }
        }
    }
}
