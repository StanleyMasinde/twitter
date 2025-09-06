use std::{
    env, fs,
    io::ErrorKind,
    process::{self, Command},
};

use crate::config::Config;

pub fn edit() {
    let config_file = dirs::home_dir()
        .unwrap()
        .join(".config/twitter_cli/config.toml");

    let editor = env::var("EDITOR")
        .or_else(|_| env::var("VISUAL"))
        .unwrap_or_else(|_| "vi".to_string());

    let status = Command::new(editor)
        .arg(config_file)
        .status()
        .expect("Failed to open the editor.");

    if status.success() {
        println!("Config edited.")
    }
}

pub fn show() {
    let config_file = dirs::home_dir()
        .unwrap()
        .join(".config/twitter_cli/config.toml");

    let file_content = fs::read_to_string(config_file).unwrap();

    let config: Config = toml::from_str(&file_content).unwrap();

    println!("{}", config);
}

pub fn init() {
    let home_dir = dirs::home_dir().expect("Home Directory not found");
    let create_dir = fs::create_dir_all(home_dir.join(".config/twitter_cli"));
    match create_dir {
        Ok(_) => println!(""),
        Err(err) => match err.kind() {
            ErrorKind::PermissionDenied => {
                eprintln!("You don't have permission to create the config dir.");
                process::exit(1)
            }
            ErrorKind::AlreadyExists => {
                eprintln!("Config directory already exists.");
            }
            _ => {
                eprintln!("An unknown error occoured.");
                process::exit(1);
            }
        },
    }
    let config_file = dirs::home_dir()
        .unwrap()
        .join(".config/twitter_cli/config.toml");

    let config = Config {
        consumer_key: "your_consumer_key".to_string(),
        consumer_secret: "your_consumer_secret".to_string(),
        access_token: "your_access_token".to_string(),
        access_secret: "your_access_secret".to_string(),
    };

    let serialized_config = match toml::to_string(&config) {
        Ok(cfg_str) => cfg_str,
        Err(_) => {
            eprintln!("Could not serialize the config.");
            process::exit(1);
        }
    };

    fs::write(config_file, serialized_config).expect("Could not write to config file.");
    println!("The config file was created please fill in your credentials.")
}
