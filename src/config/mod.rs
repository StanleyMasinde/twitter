use std::{fmt::Display, fs, process};

use dirs::home_dir;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub consumer_key: String,
    pub consumer_secret: String,
    pub access_token: String,
    pub access_secret: String,
}

impl Config {
    pub fn load() -> Self {
        let config_dir = home_dir()
            .expect("Home dir not found!")
            .join(".config/twitter_cli/config.toml");

        let data = match fs::read_to_string(config_dir) {
            Ok(data) => data,
            Err(_) => {
                eprintln!("Failed to read the config file.\nPlease run twitter config --init");
                process::exit(1)
            }
        };

        match toml::from_str::<Self>(&data) {
            Ok(cfg) => cfg,
            Err(_) => {
                eprintln!("The config file is malformed. Please run twitter config --init");
                process::exit(1)
            }
        }
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
