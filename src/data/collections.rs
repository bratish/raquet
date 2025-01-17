use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use anyhow::{Result, Context};
use directories::UserDirs;
use std::fs::File;
use std::io::{BufReader, BufWriter};
use chrono::{DateTime, Utc};
use crate::models::collection::{
    Collection, CollectionInfo, CollectionItem, 
    SavedRequest, Request, create_default_collection
};

#[derive(Clone)]
pub struct CollectionManager {
    collections_file: PathBuf,
    collections: HashMap<String, Collection>,
}

impl CollectionManager {
    pub fn new() -> Self {
        let collections_file = dirs::home_dir()
            .map(|h| h.join(".raquet").join("collections.json"))
            .unwrap_or_default();

        if !collections_file.exists() {
            // Create directory if it doesn't exist
            if let Some(parent) = collections_file.parent() {
                std::fs::create_dir_all(parent).ok();
            }

            // Create default collection
            let default_collection = create_default_collection();
            let collections = HashMap::from([
                (default_collection.info.name.clone(), default_collection)
            ]);

            // Save to file
            if let Ok(json) = serde_json::to_string_pretty(&collections) {
                std::fs::write(&collections_file, json).ok();
            }

            CollectionManager {
                collections,
                collections_file,
            }
        } else {
            let mut manager = CollectionManager {
                collections: HashMap::new(),
                collections_file,
            };
            manager.load_collections().ok();
            manager
        }
    }

    fn load_collections(&mut self) -> Result<()> {
        if self.collections_file.exists() {
            let file = File::open(&self.collections_file)?;
            let reader = BufReader::new(file);
            let collections: Vec<Collection> = serde_json::from_reader(reader).unwrap_or_default();
            
            // Convert to HashMap maintaining the order from the file
            self.collections = collections.into_iter()
                .map(|c| (c.info.name.clone(), c))
                .collect();
        }
        Ok(())
    }

    fn save_all_collections(&self) -> Result<()> {
        // Sort collections by created_at in reverse order (newest first)
        let mut collections: Vec<_> = self.collections.values().collect();
        collections.sort_by(|a, b| b.info.created_at.cmp(&a.info.created_at));
        
        let file = File::create(&self.collections_file)?;
        let writer = BufWriter::new(file);
        serde_json::to_writer_pretty(writer, &collections)?;
        Ok(())
    }

    pub fn save_collection(&mut self, collection: &Collection) -> Result<()> {
        let collection_with_timestamp = Collection {
            info: CollectionInfo {
                name: collection.info.name.clone(),
                description: collection.info.description.clone(),
                created_at: Utc::now(),
            },
            requests: collection.requests.clone(),
        };
        
        self.collections.insert(collection_with_timestamp.info.name.clone(), collection_with_timestamp);
        self.save_all_collections()
    }

    pub fn get_collections(&self) -> &HashMap<String, Collection> {
        &self.collections
    }

    pub fn get_collection(&self, name: &str) -> Option<&Collection> {
        self.collections.get(name)
    }

    pub fn save_request(&mut self, collection_name: &str, request_name: &str, method: &str, 
                       url: &str, headers: &[(String, String)], body: &str) -> Result<()> {
        if let Some(collection) = self.collections.get_mut(collection_name) {
            // Convert headers Vec to HashMap
            let headers: HashMap<String, String> = headers
                .iter()
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect();

            collection.requests.push(CollectionItem::Request(SavedRequest {
                name: request_name.to_string(),
                request: Request {
                    method: method.to_string(),
                    url: url.to_string(),
                    headers,
                    body: Some(body.to_string()),
                }
            }));
            self.save_all_collections()?;
            Ok(())
        } else {
            Err(anyhow::anyhow!("Collection not found: {}", collection_name))
        }
    }

    pub fn reload_collections(&mut self) -> Result<()> {
        self.load_collections()
    }

    pub fn delete_collection(&mut self, name: &str) {
        self.collections.remove(name);
        self.save_all_collections().ok();
    }
}

fn get_collections_dir() -> Result<PathBuf> {
    Ok(UserDirs::new()
        .context("Could not find user directory")?
        .home_dir()
        .join(".raquet"))
} 