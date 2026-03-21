use log::error;
use std::io::Write;
use std::{collections::VecDeque, env, fs::OpenOptions};

use crate::models::owntracks::{self, OwntracksPayload};

#[derive(Debug, Clone, Default)]
pub struct Buffer {
    pub checkpoint_path: String,
    pub buffer: VecDeque<owntracks::OwntracksPayload>,
    pub retry_queue: VecDeque<owntracks::OwntracksPayload>,
    pub flush_size: usize,
    // TODO: time based flush
}

impl Buffer {
    pub fn new() -> Self {
        let checkpoint_path =
            env::var("CHECKPOINT_PATH").unwrap_or_else(|_| "checkpoint.ndjson".to_string());
        let flush_size: usize = env::var("BUFFER_SIZE")
            .unwrap_or_else(|_| "10".to_string())
            .parse()
            .expect("BUFFER_SIZE must be a valid number");

        Self {
            checkpoint_path,
            buffer: VecDeque::new(),
            retry_queue: VecDeque::new(),
            flush_size,
        }
    }
    // Checkpoint current buffer so we lose minimal data
    fn flush(&self) {
        // TODO: Keep track of what was last pushed, if size is smaller
        let mut file = OpenOptions::new()
            .create(true)
            .truncate(true)
            .append(false) // TODO: Can we append latest only?
            .open(&self.checkpoint_path)
            .expect("This should open!");

        for payload in &self.buffer {
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

    pub fn enqueue(&mut self, payload: OwntracksPayload, should_flush: bool) {
        self.buffer.push_back(payload);
        // Could potentially flush a lot if we pop and push on the threshold
        if should_flush
            && self.buffer.len() > 1
            && self.buffer.len().is_multiple_of(self.flush_size)
        {
            self.flush();
        }
    }
}
