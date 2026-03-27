use log::{debug, error, info};
use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::{collections::VecDeque, fs::OpenOptions};

use crate::models::owntracks::OwntracksPayload;

const FILE_COMPACT_LINE_THRESHOLD: u16 = 50;

#[derive(Debug, Clone)]
pub struct PersistentQueue {
    pub filepath: PathBuf,
    pub queue: VecDeque<OwntracksPayload>,
    line_offset: u16,
}

impl PersistentQueue {
    pub fn new(filepath: PathBuf) -> Self {
        let offset_path = match filepath.parent() {
            Some(parent) => parent.join(".offset"),
            None => PathBuf::from("."),
        };
        debug!("setting offset path in {:?}", offset_path);
        let offset = load_offset(offset_path);

        let data = load_wal(&filepath, offset);

        Self {
            filepath,
            queue: data,
            line_offset: offset,
        }
    }

    pub fn push(&mut self, payload: OwntracksPayload) {
        self.append(payload);
        debug!("current queue size: {}", self.queue.len());
    }

    // Peek the front to try push
    pub fn pop(&self) -> Option<OwntracksPayload> {
        self.queue.front().cloned()
    }

    pub fn commit_pop(&mut self) {
        let _ = self.queue.pop_front();
        self.line_offset += 1;
        debug!("commiting offset {}", self.line_offset);
        self.write_offset();
        if self.line_offset >= FILE_COMPACT_LINE_THRESHOLD {
            self.compact();
        }
    }

    fn compact(&mut self) {
        self.commit();
        self.line_offset = 0;
        self.write_offset();
    }

    fn write_offset(&mut self) {
        let mut file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(".offset")
            .expect("This should open!");

        if let Err(e) = writeln!(file, "{}", self.line_offset) {
            error!("couldn't write to file {}", e);
        }
    }

    fn append(&mut self, payload: OwntracksPayload) {
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.filepath)
            .expect("This should open!");

        if let Ok(json) = serde_json::to_string(&payload) {
            if let Err(e) = writeln!(file, "{json}") {
                error!("couldn't write to file {}", e);
            }
        } else {
            error!("failed to deserialize payload: {payload:?}")
        }
        self.queue.push_back(payload);
    }

    fn commit(&mut self) {
        let tmp_filepath = self.filepath.with_extension("jsonl.tmp");
        let mut tmp_file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&tmp_filepath)
            .expect("This should open!");

        for payload in &self.queue {
            let result = serde_json::to_string(&payload);

            match result {
                Ok(deserialized) => {
                    if let Err(e) = writeln!(tmp_file, "{deserialized}") {
                        error!("couldn't write to file {}", e);
                    }
                }
                Err(_) => error!("failed to deserialize payload: {payload:?}"),
            }
        }

        debug!("swapping {:?} to main {:?}", &tmp_filepath, &self.filepath);
        fs::rename(&tmp_filepath, &self.filepath).expect("to work");
    }
}

fn load_wal(wal_path: &Path, offset: u16) -> VecDeque<OwntracksPayload> {
    let file = fs::File::open(wal_path);

    let mut payloads = VecDeque::new();

    match file {
        Ok(c) => {
            info!("loading WAL to memory from {wal_path:?}");
            let reader = BufReader::new(c);

            debug!("reading from line {offset}");
            for line in reader.lines().skip(offset.into()) {
                match line {
                    Ok(payload) => {
                        match serde_json::from_str::<OwntracksPayload>(&payload) {
                            Ok(data) => payloads.push_back(data),
                            Err(error) => {
                                panic!("failed to serialise checkpointed responses {error}")
                            }
                        };
                    }
                    Err(e) => {
                        error!("failed to load payload: {e}")
                    }
                }
            }

            debug!("payloads loaded from disk: {}", payloads.len());

            payloads
        }
        Err(err) => match err.kind() {
            std::io::ErrorKind::NotFound => {
                info!("no existing WAL found");
                payloads
            }
            _ => {
                panic!("failed to load wal file {err}")
            }
        },
    }
}

fn load_offset(offset_path: PathBuf) -> u16 {
    let content = match std::fs::read_to_string(offset_path) {
        Ok(content) => content,
        Err(err) => match err.kind() {
            std::io::ErrorKind::NotFound => {
                info!("no existing offset found");
                return 0;
            }
            _ => {
                error!("unexpected error: {err}");
                return 0;
            }
        },
    };

    content
        .lines()
        .next()
        .and_then(|x| x.trim().parse().ok())
        .unwrap_or(0)
}
