use std::path::PathBuf;

use crate::error::{ConfigError, Result};

pub fn get_config_file() -> Result<PathBuf> {
    let home_dir = dirs::home_dir().ok_or(ConfigError::HomeDirNotFound)?;
    Ok(home_dir.join(".config/twitter_cli/config.toml"))
}
