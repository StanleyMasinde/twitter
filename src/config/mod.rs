use std::{env::var, fmt::Display, str::FromStr};

use serde::{Deserialize, Serialize};

use crate::utils::gracefully_exit;

#[derive(Debug, Serialize, Deserialize)]
pub struct Account {
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub consumer_key: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub consumer_secret: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub access_token: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub access_secret: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub bearer_token: String,
    // oauth2.0
    pub client_id: String,
    pub client_secret: String,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Config {
    pub current_account: usize,
    pub accounts: Vec<Account>,
}

impl FromStr for Config {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let binary_name = var("CARGO_BIN_NAME").unwrap_or("twitter".to_string());

        let cfg = match toml::from_str::<Self>(s) {
            Ok(cfg) => cfg,
            Err(err) => {
                let message = format!(
                    "The config file is malformed. Please run {binary_name} config --init\n{}",
                    err
                );
                gracefully_exit(&message)
            }
        };

        Ok(cfg)
    }
}

impl Config {
    pub fn current_account(&mut self) -> &Account {
        match self.accounts.get(self.current_account) {
            Some(acc) => acc,
            None => {
                let message = format!(
                    "Account with id: {} not found. Exiting.",
                    self.current_account
                );
                gracefully_exit(&message)
            }
        }
    }
}

impl Display for Config {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let current = self.accounts.get(self.current_account).unwrap();
        write!(
            f,
            "Current Account: {}\nOAuth 2.0 Client ID: {}",
            self.current_account, current.client_id
        )
    }
}

#[cfg(test)]
#[test]
fn test_load_config() {
    let s = r#"
    current_account = 0

    [[accounts]] 
    consumer_key = "your_consumer_key"
    consumer_secret = "your_consumer_secret" 
    access_token = "your_access_token" 
    access_secret = "your_access_secret" 
    bearer_token = "your_bearer_token"
    client_id = "oauth2 client id"
    client_secret = "oauth2 client secret"
    "#;
    let test_config = Config::from_str(s).unwrap();

    assert_eq!(test_config.current_account, 0);
}

#[test]
fn oauth2_config_does_not_require_oauth1_keys() {
    let config = Config::from_str(
        "current_account = 0\n[[accounts]]\nclient_id = 'client'\nclient_secret = 'secret'",
    )
    .unwrap();
    assert!(config.accounts[0].consumer_key.is_empty());
}

#[test]
#[should_panic]
fn gracefully_fail_to_load_account() {
    let s = r#"
    current_account = 1

    [[accounts]] 
    consumer_key = "your_consumer_key"
    consumer_secret = "your_consumer_secret" 
    access_token = "your_access_token" 
    access_secret = "your_access_secret" 
    bearer_token = "your_bearer_token"
    client_id = "oauth2 client id"
    client_secret = "oauth2 client secret"
    "#;
    let test_config = Config::from_str(s).unwrap();

    assert_eq!(test_config.current_account, 0);
}
