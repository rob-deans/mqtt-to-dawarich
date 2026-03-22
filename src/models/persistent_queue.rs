use log::{debug, error, info};
use std::io::Write;
use std::{collections::VecDeque, fs::OpenOptions};

use crate::models::owntracks::OwntracksPayload;

#[derive(Debug, Clone)]
pub struct PersistentQueue {
    filepath: String,
    pub queue: VecDeque<OwntracksPayload>,
}

impl PersistentQueue {
    pub fn new(filepath: String) -> Self {
        Self {
            filepath,
            queue: VecDeque::new(),
        }
    }

    pub fn push(&mut self, payload: OwntracksPayload) {
        self.queue.push_back(payload);

        // Append to disk
        self.save();
    }

    pub fn pop(&mut self) -> Option<OwntracksPayload> {
        let payload = self.queue.pop_front();
        self.save();
        // write to disk
        payload
    }

    pub fn is_empty(&mut self) -> bool {
        self.queue.len() == 0
    }

    fn save(&mut self) {
        let mut file = OpenOptions::new()
            .create(true)
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

impl IntoIterator for PersistentQueue {
    type Item = OwntracksPayload;
    type IntoIter = std::collections::vec_deque::IntoIter<OwntracksPayload>;

    fn into_iter(self) -> Self::IntoIter {
        self.queue.into_iter()
    }
}
