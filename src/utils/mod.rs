use std::{
    env::{self, var},
    fs,
    path::PathBuf,
    process::{self, Command, ExitStatus},
    str::FromStr,
};

use dirs::home_dir;

use crate::{
    config::Config,
    schedule::Schedule,
    twitter::tweet::{Tweet, TwitterApi},
};

pub fn load_config() -> Config {
    let config_dir = home_dir()
        .expect("Home dir not found!")
        .join(".config/twitter_cli/config.toml");

    let binary_name = var("CARGO_BIN_NAME").unwrap_or("twitter".to_string());

    let data = match fs::read_to_string(config_dir) {
        Ok(data) => data,
        Err(_) => {
            let message =
                format!("Failed to read the config file.\nPlease run {binary_name} config --init");
            gracefully_exit(&message)
        }
    };
    match Config::from_str(&data) {
        Ok(cfg) => cfg,
        Err(_) => {
            gracefully_exit("Failed to load config. Try and run {binary_name} config --init");
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
        .unwrap_or_else(|_| "vi".to_string());

    #[cfg(windows)]
    let editor = "notepad".to_string();

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
            println!("> Windows does not support POSIX permissions. Skipping check.");
        }
    }
}

pub(crate) fn gracefully_exit(message: &str) -> ! {
    println!("{message}");
    process::exit(1)
}

pub(crate) async fn send_due_tweets() {
    let schedule = Schedule::default();
    let due_tweets = schedule.due();
    for (index, due_tweet) in due_tweets.iter().enumerate() {
        println!("> Sending tweet {}/{}", index, due_tweets.len());
        let mut tweet = Tweet::from_str(&due_tweet.body).unwrap();
        let api_res = tweet.create().await;
        match api_res {
            Ok(res) => println!("{}", res.content),
            Err(err) => {
                // We mark this as failed.
                // For now let us just log this

                eprintln!("{}", err.message)
            }
        }
    }
}
