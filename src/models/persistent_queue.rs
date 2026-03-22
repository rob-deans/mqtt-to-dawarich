use log::{debug, error, info};
use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::{collections::VecDeque, fs::OpenOptions};

use crate::models::owntracks::OwntracksPayload;

#[derive(Debug, Clone)]
pub struct PersistentQueue {
    pub filepath: String,
    pub queue: VecDeque<OwntracksPayload>,
}

impl PersistentQueue {
    pub fn new(filepath: String) -> Self {
        let data = load_wal(&filepath);

        Self {
            filepath,
            queue: data,
        }
    }

    pub fn push(&mut self, payload: OwntracksPayload) {
        self.append(&payload);
        self.queue.push_back(payload);
        debug!("Queue size {}", self.queue.len());
    }

    pub fn pop(&mut self) -> Option<OwntracksPayload> {
        let payload = self.queue.pop_front();
        self.save();
        payload
    }

    pub fn is_empty(&mut self) -> bool {
        self.queue.len() == 0
    }

    fn append(&self, payload: &OwntracksPayload) {
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.filepath)
            .expect("This should open!");

        if let Ok(json) = serde_json::to_string(&payload) {
            if let Err(e) = writeln!(file, "{json}") {
                error!("Couldn't write to file {}", e);
            }
        } else {
            error!("Failed to deserialize payload: {payload:?}")
        }
    }

    fn save(&self) {
        let mut file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&self.filepath)
            .expect("This should open!");

        for payload in &self.queue {
            let result = serde_json::to_string(&payload);

            match result {
                Ok(deserialized) => {
                    if let Err(e) = writeln!(file, "{deserialized}") {
                        error!("Couldn't write to file {}", e);
                    }
                }
                Err(_) => error!("Failed to deserialize payload: {payload:?}"),
            }
        }
    }
}

pub fn load_wal(wal_path: &String) -> VecDeque<OwntracksPayload> {
    let file = fs::File::open(wal_path);

    let mut payloads = VecDeque::new();

    match file {
        Ok(c) => {
            info!("Loading WAL to memory from {wal_path}");
            let reader = BufReader::new(c);

            for line in reader.lines() {
                match line {
                    Ok(payload) => {
                        match serde_json::from_str::<OwntracksPayload>(&payload) {
                            Ok(data) => payloads.push_back(data),
                            Err(error) => {
                                panic!("Failed to serialise checkpointed responses {error}")
                            }
                        };
                    }
                    Err(e) => {
                        error!("Failed to load payload: {e}")
                    }
                }
            }

            payloads
        }
        // TODO: Should handle all errors
        Err(err) => match err.kind() {
            std::io::ErrorKind::NotFound => {
                info!("no existing WAL found");
                payloads
            }
            _ => {
                panic!("something else: {err}")
            }
        },
    }
}
