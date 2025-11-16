use std::{fs, io::ErrorKind, process};

use crate::{
    config::{Account, Config},
    utils,
};

pub fn edit() {
    let config_file = utils::get_config_file();

    let status = utils::open_editor(&config_file);

    if status.success() {
        println!("Config edited.")
    }
}

pub fn show() {
    let binary_name = env!("CARGO_BIN_NAME");
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
                    eprintln!(
                        "Failed to set permissions\nPlease run chmod 700 {}",
                        config_dir.to_str().unwrap()
                    )
                }
            }
        }
        Err(err) => match err.kind() {
            ErrorKind::PermissionDenied => {
                eprintln!("You don't have permission to create the config dir.");
                process::exit(1)
            }
            ErrorKind::AlreadyExists => {
                eprintln!("Config directory already exists.");
            }
            _ => {
                eprintln!("An unknown error occurred.");
                process::exit(1);
            }
        },
    }

    let config_file = utils::get_config_file();

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let file_perms = fs::Permissions::from_mode(0o600);
        if fs::set_permissions(&config_file, file_perms).is_ok() {
            println!("> Config file permissions set to 600")
        } else {
            eprintln!(
                "Failed to set permissions for config file\nPlease run chmod 600 {}",
                config_file.to_str().unwrap()
            )
        }
    }

    let account = Account {
        consumer_key: "your_consumer_key".to_string(),
        consumer_secret: "your_consumer_secret".to_string(),
        access_token: "your_access_token".to_string(),
        access_secret: "your_access_secret".to_string(),
        bearer_token: "your_bearer_token".to_string(),
    };

    let config = Config {
        current_account: 0,
        accounts: vec![account],
    };

    let serialized_config = match toml::to_string(&config) {
        Ok(cfg_str) => cfg_str,
        Err(_) => {
            eprintln!("Could not serialize the config.");
            process::exit(1);
        }
    };

    fs::write(config_file, serialized_config).expect("Could not write to config file.");
    println!("> The config file was created please fill in your credentials.")
}

pub fn validate() {
    utils::check_permissions(&utils::get_config_dir(), true);
    utils::check_permissions(&utils::get_config_file(), false);
    println!("> Validation complete. Please check for any warnings and address them.")
}
