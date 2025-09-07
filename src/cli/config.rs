use std::{env, fs, io::ErrorKind, process::Command};

use crate::{
    config::Config,
    error::{ConfigError, Result, TwitterError},
    utils,
};

pub fn edit() -> Result<()> {
    let config_file = utils::get_config_file()?;

    let editor = env::var("EDITOR")
        .or_else(|_| env::var("VISUAL"))
        .unwrap_or_else(|_| "vi".to_string());

    let status = Command::new(&editor)
        .arg(&config_file)
        .status()
        .map_err(|e| ConfigError::EditorFailed {
            editor: editor.clone(),
            source: e,
        })?;

    if status.success() {
        println!("Config edited successfully.");
    } else {
        eprintln!("Editor exited with non-zero status code.");
    }

    Ok(())
}

pub fn show() -> Result<()> {
    let config_file = utils::get_config_file()?;
    let file_content = fs::read_to_string(&config_file).map_err(|e| ConfigError::ReadFailed {
        path: config_file.to_string_lossy().to_string(),
        source: e,
    })?;

    let config: Config =
        toml::from_str(&file_content).map_err(TwitterError::TomlDeserializeError)?;

    println!("{config}");
    Ok(())
}

pub fn init() -> Result<()> {
    let home_dir = dirs::home_dir().ok_or(ConfigError::HomeDirNotFound)?;
    let config_dir = home_dir.join(".config/twitter_cli");

    match fs::create_dir_all(&config_dir) {
        Ok(_) => println!("Created config directory: {}", config_dir.display()),
        Err(err) => match err.kind() {
            ErrorKind::PermissionDenied => {
                return Err(ConfigError::WriteFailed {
                    path: config_dir.to_string_lossy().to_string(),
                    source: err,
                }
                .into());
            }
            ErrorKind::AlreadyExists => {
                println!("Config directory already exists: {}", config_dir.display());
            }
            _ => {
                return Err(ConfigError::WriteFailed {
                    path: config_dir.to_string_lossy().to_string(),
                    source: err,
                }
                .into());
            }
        },
    }

    let config_file = utils::get_config_file()?;

    let config = Config {
        consumer_key: "your_consumer_key".to_string(),
        consumer_secret: "your_consumer_secret".to_string(),
        access_token: "your_access_token".to_string(),
        access_secret: "your_access_secret".to_string(),
    };

    let serialized_config =
        toml::to_string(&config).map_err(TwitterError::TomlSerializeError)?;

    fs::write(&config_file, serialized_config).map_err(|e| ConfigError::WriteFailed {
        path: config_file.to_string_lossy().to_string(),
        source: e,
    })?;

    println!("Config file created at: {}", config_file.display());
    println!("Please edit the file and fill in your Twitter API credentials.");
    Ok(())
}
