use std::{env, process::Command};

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
