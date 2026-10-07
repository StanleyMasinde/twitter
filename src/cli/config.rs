use std::{
    env::var,
    fs,
    io::{ErrorKind, Write},
};

use crate::{
    config::{Account, Config},
    utils::{self, gracefully_exit},
};

pub fn edit() {
    let config_file = utils::get_config_file();
    match config_file.try_exists() {
        Ok(false) => init(),
        Ok(true) => {}
        Err(err) => gracefully_exit(&format!("Failed to check the config file: {err}")),
    }

    let status = utils::open_editor(&config_file);
    if !status.success() {
        gracefully_exit("The editor exited unsuccessfully. Config editing failed.");
    }
    if !config_file.is_file() {
        gracefully_exit("Config editing failed: the config file does not exist.");
    }
    println!("Config edited.");
}

pub fn show() {
    let binary_name = var("CARGO_BIN_NAME").unwrap_or("twitter".to_string());
    let config_file = utils::get_config_file();
    if let Ok(file_content) = fs::read_to_string(config_file) {
        if let Ok(config) = toml::from_str::<Config>(&file_content) {
            println!("{}", config);
        } else {
            eprintln!("Invalid config format.\nPlease run {binary_name} config --init")
        }
    } else {
        eprintln!("Failed to read to config file.\nPlease run {binary_name} config --init")
    }
}

pub fn init() {
    dirs::home_dir().expect("Home Directory not found");
    let config_dir = utils::get_config_dir();
    let create_dir = fs::create_dir_all(&config_dir);
    match create_dir {
        Ok(_) => {
            println!("> Created home config dir.");

            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let perms = fs::Permissions::from_mode(0o700);
                if fs::set_permissions(&config_dir, perms).is_ok() {
                    println!("> Config dir permissions set to 700")
                } else {
                    use crate::utils::gracefully_exit;

                    gracefully_exit("Failed to set permissions\nPlease run chmod 700 {}")
                }
            }
        }
        Err(err) => match err.kind() {
            ErrorKind::PermissionDenied => {
                gracefully_exit("You don't have permission to create the config dir.")
            }
            ErrorKind::AlreadyExists => {
                eprintln!();
                gracefully_exit("Config directory already exists")
            }
            _ => gracefully_exit("An unknown error occurred."),
        },
    }

    let config_file = utils::get_config_file();
    let mut options = fs::OpenOptions::new();
    options.write(true).create(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let mut file = options
        .open(&config_file)
        .expect("Could not create config file.");

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let file_perms = fs::Permissions::from_mode(0o600);
        if file.set_permissions(file_perms).is_ok() {
            println!("> Config file permissions set to 600")
        } else {
            let message = format!(
                "Failed to set permissions for the config file.\nPlease run chmod 600 {}",
                config_file.to_str().unwrap()
            );

            gracefully_exit(&message)
        }
    }

    let account = Account {
        consumer_key: String::new(),
        consumer_secret: String::new(),
        access_token: String::new(),
        access_secret: String::new(),
        bearer_token: "your_bearer_token".to_string(),
        client_id: "your_oauth2_client_id".to_string(),
        client_secret: "your_oauth2_client_secret".to_string(),
    };

    let config = Config {
        current_account: 0,
        accounts: vec![account],
    };

    let serialized_config = match toml::to_string(&config) {
        Ok(cfg_str) => cfg_str,
        Err(_) => gracefully_exit("Could not serialize the config."),
    };

    file.set_len(0).expect("Could not clear config file.");
    file.write_all(serialized_config.as_bytes())
        .expect("Could not write to config file.");
    println!("> The config file was created please fill in your credentials.")
}

pub fn validate() {
    utils::check_permissions(&utils::get_config_dir(), true);
    utils::check_permissions(&utils::get_config_file(), false);
    println!("> Validation complete. Please check for any warnings and address them.")
}
