use std::path::PathBuf;

pub fn get_config_file() -> PathBuf {
    dirs::home_dir()
        .unwrap()
        .join(".config/twitter_cli/config.toml")
}
