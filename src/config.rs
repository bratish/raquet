use config::{Config, ConfigError, File};
use directories::UserDirs;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

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
    pub default_headers: std::collections::HashMap<String, String>,
    #[serde(default = "default_timeout")]
    pub timeout_seconds: u64,
    #[serde(default = "default_max_response_size")]
    pub max_response_size: usize,
    #[serde(default = "default_history_size")]
    pub history_size: usize,
    #[serde(default)]
    pub default_url: String,
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
        let mut default_headers = std::collections::HashMap::new();
        
        // Common default headers
        default_headers.insert("Random-Token".to_string(), uuid::Uuid::new_v4().to_string());
        default_headers.insert("Content-Type".to_string(), "application/json".to_string());
        default_headers.insert("Content-Length".to_string(), "<calculated>".to_string());
        default_headers.insert("Host".to_string(), "<host of the machine>".to_string());
        default_headers.insert("User-Agent".to_string(), "Raquet".to_string());
        default_headers.insert("Accept".to_string(), "*/*".to_string());
        default_headers.insert("Accept-Encoding".to_string(), "gzip, deflate, br".to_string());
        default_headers.insert("Connection".to_string(), "keep-alive".to_string());

        Self {
            default_headers,
            timeout_seconds: default_timeout(),
            max_response_size: default_max_response_size(),
            history_size: default_history_size(),
            default_url: String::new(),
        }
    }
}

impl AppConfig {
    pub fn load() -> Result<Self, Error> {
        let config_path = get_config_path()?;
        
        // Create default config if it doesn't exist
        if !config_path.exists() {
            Self::create_default_config(&config_path)?;
        }

        let config = Config::builder()
            .add_source(File::from(config_path))
            .build()?;

        Ok(config.try_deserialize()?)
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