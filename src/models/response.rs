use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ResponseData {
    pub status: Option<u16>,
    pub status_text: Option<String>,
    pub headers: HashMap<String, String>,
    pub body: String,
    pub time_ms: u128,
    pub size_bytes: usize,
}

#[derive(Debug, Clone)]
pub struct ResponseMetadata {
    pub status: u16,
    pub status_text: String,
    pub time_ms: u128,
    pub size_bytes: usize,
    pub response_headers: HashMap<String, String>,
    pub timestamp: DateTime<Utc>,
} 