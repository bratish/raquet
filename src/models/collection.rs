use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Collection {
    pub info: CollectionInfo,
    #[serde(default)]
    pub requests: Vec<CollectionItem>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SavedRequest {
    pub name: String,
    pub request: Request,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Request {
    pub method: String,
    pub url: String,
    #[serde(default)]
    pub headers: HashMap<String, String>,
    pub body: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum CollectionItem {
    Request(SavedRequest),
    Folder(Folder),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Folder {
    pub name: String,
    pub item: Vec<CollectionItem>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Response {}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CollectionInfo {
    pub name: String,
    pub description: String,
    pub created_at: DateTime<Utc>,
}

// Function to create default collection
pub fn create_default_collection() -> Collection {
    Collection {
        info: CollectionInfo {
            name: "Default Collection".to_string(),
            description: "Default collection for saved requests".to_string(),
            created_at: chrono::Utc::now(),
        },
        requests: Vec::new(),
    }
} 