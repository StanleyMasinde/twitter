use std::{fmt::Display, process, str::FromStr};

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Account {
    pub consumer_key: String,
    pub consumer_secret: String,
    pub access_token: String,
    pub access_secret: String,
    pub bearer_token: String,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Config {
    pub current_account: usize,
    pub accounts: Vec<Account>,
}

impl FromStr for Config {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let binary_name = env!("CARGO_BIN_NAME");

        let cfg = match toml::from_str::<Self>(s) {
            Ok(cfg) => cfg,
            Err(err) => {
                eprintln!("The config file is malformed. Please run {binary_name} config --init");
                eprintln!("{}", err);
                process::exit(1)
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
                eprintln!(
                    "Account with id: {} not found. Exiting.",
                    self.current_account
                );
                process::exit(1)
            }
        }
    }
}

impl Display for Config {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let current = self.accounts.get(self.current_account).unwrap();
        write!(
            f,
            "Current Account: {} \nConsumer Key: {}\nConsumer Secret: {}\nAccess Token: {}\nAccess Token Secret: {}",
            self.current_account,
            current.consumer_key,
            current.consumer_secret,
            current.access_token,
            current.access_secret
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
    "#;
    let test_config = Config::from_str(s).unwrap();

    assert_eq!(test_config.current_account, 0);
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
    "#;
    let test_config = Config::from_str(s).unwrap();

    assert_eq!(test_config.current_account, 0);
}
