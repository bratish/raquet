use crate::app::HttpMethod;
use chrono::{DateTime, Utc};
use directories::UserDirs;
use serde::{Deserialize, Serialize};
use std::{
    fs::{self, File},
    io::{self, BufReader, BufWriter},
    path::PathBuf,
};

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct HistoryEntry {
    pub timestamp: DateTime<Utc>,
    pub request: RequestData,
    pub response: Option<ResponseData>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct RequestData {
    pub url: String,
    pub method: String,
    pub headers: std::collections::HashMap<String, String>,
    pub body: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct ResponseData {
    pub status: Option<u16>,
    pub status_text: Option<String>,
    pub headers: std::collections::HashMap<String, String>,
    pub body: String,
    pub time_ms: u128,
    pub size_bytes: usize,
}

#[derive(Debug)]
pub enum HistoryError {
    Io(io::Error),
    Json(serde_json::Error),
    NoUserDir,
}

impl From<io::Error> for HistoryError {
    fn from(err: io::Error) -> Self {
        HistoryError::Io(err)
    }
}

impl From<serde_json::Error> for HistoryError {
    fn from(err: serde_json::Error) -> Self {
        HistoryError::Json(err)
    }
}

#[derive(Clone)]
pub struct History {
    entries: Vec<HistoryEntry>,
    max_entries: usize,
    file_path: PathBuf,
}

impl History {
    pub fn new(max_entries: usize) -> Result<Self, HistoryError> {
        let file_path = get_history_path()?;
        let entries = if file_path.exists() {
            let file = File::open(&file_path)?;
            let reader = BufReader::new(file);
            serde_json::from_reader(reader).unwrap_or_default()
        } else {
            Vec::new()
        };

        Ok(Self {
            entries,
            max_entries,
            file_path,
        })
    }

    pub fn add_entry(
        &mut self,
        url: String,
        method: HttpMethod,
        headers: std::collections::HashMap<String, String>,
        body: Option<String>,
        response: Option<ResponseData>,
    ) -> Result<(), HistoryError> {
        let entry = HistoryEntry {
            timestamp: Utc::now(),
            request: RequestData {
                url,
                method: method.as_str().to_string(),
                headers,
                body,
            },
            response,
        };

        self.entries.push(entry);

        // Keep only the most recent entries
        if self.entries.len() > self.max_entries {
            self.entries.remove(0);
        }

        self.save()
    }

    pub fn get_entries(&self) -> &[HistoryEntry] {
        &self.entries
    }

    fn save(&self) -> Result<(), HistoryError> {
        // Create directory if it doesn't exist
        if let Some(parent) = self.file_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let file = File::create(&self.file_path)?;
        let writer = BufWriter::new(file);
        serde_json::to_writer_pretty(writer, &self.entries)?;
        Ok(())
    }
}

fn get_history_path() -> Result<PathBuf, HistoryError> {
    UserDirs::new()
        .ok_or(HistoryError::NoUserDir)
        .map(|dirs| dirs.home_dir().join(".raquet").join("history.json"))
} 