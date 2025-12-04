use std::{
    env, fs,
    path::PathBuf,
    process::{self, Command, ExitStatus},
    str::FromStr,
};

use dirs::home_dir;

use crate::config::Config;

pub fn load_config() -> Config {
    let config_dir = home_dir()
        .expect("Home dir not found!")
        .join(".config/twitter_cli/config.toml");

    let binary_name = env!("CARGO_BIN_NAME");

    let data = match fs::read_to_string(config_dir) {
        Ok(data) => data,
        Err(_) => {
            eprintln!("Failed to read the config file.\nPlease run {binary_name} config --init");
            process::exit(1)
        }
    };
    match Config::from_str(&data) {
        Ok(cfg) => cfg,
        Err(_) => {
            eprintln!("Failed to load config. Try and run {binary_name} config --init");
            process::exit(1)
        }
    }
}

pub fn get_config_dir() -> PathBuf {
    dirs::home_dir()
        .expect("Home directory missing!")
        .join(".config/twitter_cli")
}

pub fn get_config_file() -> PathBuf {
    dirs::home_dir()
        .expect("Home directory missing!")
        .join(".config/twitter_cli/config.toml")
}

pub fn open_editor(file: &PathBuf) -> ExitStatus {
    #[cfg(unix)]
    let editor = env::var("EDITOR")
        .or_else(|_| env::var("VISUAL"))
        .unwrap_or_else(|_| {
            if cfg!(windows) {
                return "notepad".to_string();
            } else {
                "vi".to_string()
            }
        });

    Command::new(editor)
        .arg(file)
        .status()
        .expect("Failed to open the editor.")
}

pub fn check_permissions(path: &PathBuf, is_dir: bool) {
    if let Ok(metadata) = fs::metadata(path) {
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mode = metadata.permissions().mode() & 0o777;

            let expected = if is_dir { 0o700 } else { 0o600 };

            if mode != expected {
                println!(
                    "⚠️  Permissions for {:?} are {:o}, expected {:o}",
                    path, mode, expected
                );
                println!("Run chmod {:o} {:?}", expected, path)
            }
        }

        #[cfg(windows)]
        {
            println!("> Windows does not support permissions.");
        }
    }
}
