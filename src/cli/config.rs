use std::{env, fs, process::Command};

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
