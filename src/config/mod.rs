use std::{fmt::Display, fs};

use dirs::home_dir;
use serde::{Deserialize, Serialize};

use crate::error::{ConfigError, Result, TwitterError};

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub consumer_key: String,
    pub consumer_secret: String,
    pub access_token: String,
    pub access_secret: String,
}

impl Config {
    pub fn load() -> Result<Self> {
        let config_dir = home_dir()
            .ok_or(ConfigError::HomeDirNotFound)?
            .join(".config/twitter_cli/config.toml");

        let data = fs::read_to_string(&config_dir).map_err(|e| ConfigError::ReadFailed {
            path: config_dir.to_string_lossy().to_string(),
            source: e,
        })?;

        let config: Config =
            toml::from_str(&data).map_err(|e| TwitterError::TomlDeserializeError(e))?;

        // Validate that all required fields are present and not default values
        if config.consumer_key == "your_consumer_key" {
            return Err(ConfigError::MissingField {
                field: "consumer_key".to_string(),
            }
            .into());
        }
        if config.consumer_secret == "your_consumer_secret" {
            return Err(ConfigError::MissingField {
                field: "consumer_secret".to_string(),
            }
            .into());
        }
        if config.access_token == "your_access_token" {
            return Err(ConfigError::MissingField {
                field: "access_token".to_string(),
            }
            .into());
        }
        if config.access_secret == "your_access_secret" {
            return Err(ConfigError::MissingField {
                field: "access_secret".to_string(),
            }
            .into());
        }

        Ok(config)
    }
}

impl Display for Config {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Consumer Key: {}\nConsumer Secret: {}\nAccess Token: {}\nAccess Token Secret: {}",
            self.consumer_key, self.consumer_secret, self.access_token, self.access_secret
        )
    }
}
