use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use anyhow::{Result, Context};
use directories::UserDirs;
use std::fs::File;
use std::io::{BufReader, BufWriter};
use chrono::{DateTime, Utc};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Collection {
    pub info: CollectionInfo,
    pub item: Vec<CollectionItem>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CollectionInfo {
    pub name: String,
    pub description: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(untagged)]
pub enum CollectionItem {
    Request(RequestItem),
    Folder(FolderItem),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RequestItem {
    pub name: String,
    pub request: Request,
    pub response: Vec<Response>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FolderItem {
    pub name: String,
    pub item: Vec<CollectionItem>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Request {
    pub method: String,
    pub header: Vec<Header>,
    pub body: RequestBody,
    pub url: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Header {
    pub key: String,
    pub value: String,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct RequestBody {
    #[serde(default)]
    pub mode: String,
    #[serde(default)]
    pub raw: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Response {
    // Add response fields as needed
}

#[derive(Clone)]
pub struct CollectionManager {
    collections_file: PathBuf,
    collections: HashMap<String, Collection>,
}

impl CollectionManager {
    pub fn new() -> Result<Self> {
        let collections_dir = get_collections_dir()?;
        fs::create_dir_all(&collections_dir)?;
        let collections_file = collections_dir.join("collections.json");
        
        let mut manager = Self {
            collections_file,
            collections: HashMap::new(),
        };
        
        manager.load_collections()?;
        
        // Create default collection if none exists
        if manager.collections.is_empty() {
            let default_collection = Collection {
                info: CollectionInfo {
                    name: "Default Collection".to_string(),
                    description: "Default collection for saved requests".to_string(),
                    created_at: Utc::now(),
                },
                item: Vec::new(),
            };
            manager.save_collection(&default_collection)?;
        }
        
        Ok(manager)
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
            item: collection.item.clone(),
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
        let headers = headers.iter().map(|(k, v)| Header {
            key: k.clone(),
            value: v.clone(),
        }).collect();

        let request_item = RequestItem {
            name: request_name.to_string(),
            request: Request {
                method: method.to_string(),
                header: headers,
                body: RequestBody {
                    mode: "raw".to_string(),
                    raw: body.to_string(),
                },
                url: url.to_string(),
            },
            response: Vec::new(),
        };

        if let Some(collection) = self.collections.get_mut(collection_name) {
            collection.item.push(CollectionItem::Request(request_item));
            self.save_all_collections()?;
            Ok(())
        } else {
            Err(anyhow::anyhow!("Collection not found: {}", collection_name))
        }
    }

    pub fn reload_collections(&mut self) -> Result<()> {
        self.load_collections()
    }
}

fn get_collections_dir() -> Result<PathBuf> {
    Ok(UserDirs::new()
        .context("Could not find user directory")?
        .home_dir()
        .join(".raquet"))
} 