use config::ConfigError;
use directories::UserDirs;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::collections::HashMap;
use log::{debug, info, warn};

#[derive(Debug)]
pub enum Error {
    Config(ConfigError),
    Io(std::io::Error),
    Toml(toml::ser::Error),
}

impl From<ConfigError> for Error {
    fn from(err: ConfigError) -> Self {
        Error::Config(err)
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::Io(err)
    }
}

impl From<toml::ser::Error> for Error {
    fn from(err: toml::ser::Error) -> Self {
        Error::Toml(err)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AppConfig {
    #[serde(default)]
    pub app: AppSettings,
    #[serde(default)]
    pub default_headers: HashMap<String, String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AppSettings {
    #[serde(default = "default_timeout")]
    pub timeout_seconds: u64,
    
    #[serde(default = "default_max_response_size")]
    pub max_response_size: usize,
    
    #[serde(default = "default_history_size")]
    pub history_size: usize,
    
    #[serde(default)]
    pub default_url: String,
    
    #[serde(default)]
    pub default_headers: HashMap<String, String>,
}

fn default_timeout() -> u64 {
    30
}

fn default_max_response_size() -> usize {
    10 * 1024 * 1024 // 10MB
}

fn default_history_size() -> usize {
    100
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            app: AppSettings::default(),
            default_headers: HashMap::new(),
        }
    }
}

impl Default for AppSettings {
    fn default() -> Self {
        let mut default_headers = HashMap::new();
        default_headers.insert("Content-Type".to_string(), "application/json".to_string());
        default_headers.insert("Accept".to_string(), "*/*".to_string());
        default_headers.insert("Accept-Encoding".to_string(), "gzip, deflate, br".to_string());
        default_headers.insert("User-Agent".to_string(), "raquet/1.0".to_string());
        default_headers.insert("Connection".to_string(), "keep-alive".to_string());
        default_headers.insert("Content-Length".to_string(), "<calculated>".to_string());
        default_headers.insert("Host".to_string(), "<from url>".to_string());
        default_headers.insert("Random-Token".to_string(), "<generated>".to_string());

        Self {
            timeout_seconds: default_timeout(),
            max_response_size: default_max_response_size(),
            history_size: default_history_size(),
            default_url: String::new(),
            default_headers,
        }
    }
}

impl AppConfig {
    pub fn load() -> Result<Self, Error> {
        let config_path = get_config_path()?;
        debug!("Loading config from: {:?}", config_path);
        
        if config_path.exists() {
            let content = std::fs::read_to_string(&config_path)?;
            debug!("Config content:\n{}", content);
            
            match toml::from_str::<AppConfig>(&content) {
                Ok(mut config) => {
                    if !config.default_headers.is_empty() {
                        config.app.default_headers.extend(config.default_headers.clone());
                    }
                    
                    debug!("Default headers: {:?}", config.app.default_headers);
                    if config.app.default_headers.is_empty() {
                        warn!("No default headers found in config");
                    }
                    Ok(config)
                }
                Err(e) => {
                    warn!("Error parsing config: {}", e);
                    Ok(Self::default())
                }
            }
        } else {
            info!("Config file not found, creating default");
            Self::create_default_config(&config_path)?;
            Ok(Self::default())
        }
    }

    fn create_default_config(path: &Path) -> Result<(), Error> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let default_config = Self::default();
        let toml = toml::to_string_pretty(&default_config)?;
        std::fs::write(path, toml)?;
        Ok(())
    }

    pub fn save(&self) -> Result<(), Error> {
        let config_path = get_config_path()?;
        let toml = toml::to_string_pretty(&self)?;
        std::fs::write(config_path, toml)?;
        Ok(())
    }
}

fn get_config_path() -> Result<PathBuf, Error> {
    UserDirs::new()
        .ok_or_else(|| {
            Error::Config(ConfigError::NotFound(
                "Could not find user directory".into()
            ))
        })
        .map(|dirs| dirs.home_dir().join(".raquet").join("config.toml"))
} 