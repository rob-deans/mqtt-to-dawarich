use log::{debug, error, info};
use models::{dawarich, owntracks};
use rumqttc::{Client, Event, Incoming, MqttOptions, QoS};
use std::collections::VecDeque;
use std::fs::OpenOptions;
use std::io::{BufRead, BufReader, Write};
use std::time::Duration;
use std::{env, fs};

use crate::models::buffer::Buffer;

pub mod models;

// Either a) always push to buffer then loop through
// b) only push when failed

// in-memory buffer if connection fails
// checkpoint buffer i.e. append to an ndjson file
// checkpoint based on time or size (memory/len)
// on start up -> load (attempt) the ndjson -> push to api -> then listen - done

// on startup reads the checkpoint then pushes to dawarich. if fails, bets off
fn load_checkpoint(checkpoint_path: &String) -> Option<Vec<owntracks::OwntracksPayload>> {
    let file = fs::File::open(checkpoint_path);

    let mut buffer: Vec<owntracks::OwntracksPayload> = [].to_vec();

    match file {
        Ok(c) => {
            info!("Reading latest checkpoint data to push");
            let reader = BufReader::new(c);

            for line in reader.lines() {
                // TODO: Proper match on this
                let payload = line.expect("cannot read line");

                match serde_json::from_str::<owntracks::OwntracksPayload>(&payload) {
                    Ok(data) => buffer.push(data),
                    Err(error) => {
                        panic!("Failed to serialise checkpointed responses {error}")
                    }
                };
            }

            Some(buffer)
        }
        // TODO: Should handle all errors
        Err(err) => match err.kind() {
            std::io::ErrorKind::NotFound => {
                info!("No checkpoint found. Not uploading.");
                None
            }
            _ => {
                panic!("something else: {err}")
            }
        },
    }
}

// Checkpoint current buffer so we lose minimal data
fn flush(checkpoint_path: &String, buffer: &VecDeque<owntracks::OwntracksPayload>) {
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(checkpoint_path)
        .expect("This should open!!");

    for payload in buffer {
        let result = serde_json::to_string(&payload);

        match result {
            Ok(deserialized) => {
                if let Err(e) = writeln!(file, "{deserialized}") {
                    error!("Couldn't write to file {}", e);
                }
            }
            Err(_) => error!("SOMETHING HAPPENED WHEN DESERIALIZING"),
        }
    }
}

// fn push_to_dawarich(
//     client: &reqwest::blocking::Client,
//     dawarich_config: &dawarich::DawarichConfig,
//     payload: owntracks::OwntracksPayload,
//     error_buffer: &VecDeque<owntracks::OwntracksPayload>,
//     checkpoint_path: &String,
// ) {
//     let response = client
//         .post(dawarich_config.endpoint.clone())
//         .json(&payload)
//         .bearer_auth(dawarich_config.api_key.clone())
//         .send();

//     match response {
//         Ok(resp) => debug!("Response: {resp:?}"),
//         Err(err) => {
//             println!("We made it here");
//             error!("Request failed with error: {err:?}");
//             println!("and here");
//             error_buffer.push_back(payload);
//             // add to bufferf
//             // TODO: Add config for this
//             if error_buffer.len() > 1 {
//                 flush(checkpoint_path, error_buffer);
//             }
//         }
//     }
// }

fn main() {
    env_logger::init();

    let checkpoint_path =
        env::var("CHECKPOINT_PATH").unwrap_or_else(|_| "checkpoint.ndjson".to_string());
    // let buffer_size: usize = env::var("BUFFER_SIZE")
    //     .unwrap_or_else(|_| "50".to_string())
    //     .parse()
    //     .expect("BUFFER_SIZE must be a valid number");

    let mut dawarich = dawarich::Dawarich::from_env();

    // let mut b = Buffer::new();

    // let d_client = reqwest::blocking::Client::new();

    // TODO: Should this go to the buffer then handled in the normal flow?
    if let Some(data) = load_checkpoint(&checkpoint_path) {
        info!("Pushing {} point(s) to dawarich", data.len());

        for payload in data {
            println!("{payload:?}");
            // push_to_dawarich(
            //     &d_client,
            //     &dawarich_config,
            //     payload,
            //     &error_buffer,
            // );
        }
        // On success truncate the file
        debug!(
            "Attempting to remove old checkpoint path at: {}",
            &checkpoint_path
        );
        match fs::remove_file(&checkpoint_path) {
            Ok(_) => {
                info!("Successfully deleted checkpoint at: {}", &checkpoint_path)
            }
            Err(err) => error!("Failed to delete checkpoint at {}: {err}", &checkpoint_path),
        }
    }

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

    // info!(
    //     "Sending data to Dawarich at {}:{}",
    //     dawarich.endpoint, dawarich.port
    // );

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
                    error!("Failed to resubscribe! {:?}", err);
                }
            }
            Ok(notif) => match notif {
                Event::Incoming(Incoming::Publish(package)) => {
                    match serde_json::from_slice::<owntracks::OwntracksPayload>(&package.payload) {
                        Ok(data) => {
                            if data._type != "location" {
                                debug!(
                                    "Ignoring non-location payload type. Payload was: {:?}",
                                    data._type
                                );
                                continue;
                            }

                            dawarich.push_and_try_flush(data);

                            // b.enqueue(data.clone(), true);
                            // // buffer.push_back(data);

                            // for _ in 0..b.buffer.len() {
                            //     if let Some(payload) = b.buffer.pop_front() {
                            //         dawarich.push(payload);
                            //         // let response = d_client
                            //         //     .post(dawarich.endpoint.clone())
                            //         //     .json(&payload)
                            //         //     .bearer_auth(dawarich.api_key.clone())
                            //         //     .send();

                            //         // match response {
                            //         //     Ok(resp) => {
                            //         //         debug!("Response: {resp:?}");
                            //         //     }
                            //         //     // TODO: Do we skip if network error and insert a delay
                            //         //     // to avoid spamming when it's probably not going to work
                            //         //     Err(err) => {
                            //         //         error!("Request failed with error: {err:?}");
                            //         //         // Re-add to buffer
                            //         //         println!("{}", b.buffer.len());
                            //         //         b.enqueue(data.clone(), false);
                            //         //         // break;
                            //         //     }
                            //         // }
                            //     }
                            // }
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
