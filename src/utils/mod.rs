use std::{
    env,
    path::PathBuf,
    process::{Command, ExitStatus},
};

pub fn get_config_file() -> PathBuf {
    dirs::home_dir()
        .unwrap()
        .join(".config/twitter_cli/config.toml")
}

pub fn open_editor(file: &PathBuf) -> ExitStatus {
    let editor = env::var("EDITOR")
        .or_else(|_| env::var("VISUAL"))
        .unwrap_or_else(|_| "vi".to_string());

    Command::new(editor)
        .arg(file)
        .status()
        .expect("Failed to open the editor.")
}
