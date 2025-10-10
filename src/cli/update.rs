use futures_util::StreamExt;
use std::{
    env,
    process::{self, Command},
};
use tokio::{fs::File, io::AsyncWriteExt};

pub async fn run() {
    let binary_name = "twitter";
    let os = env::consts::OS;
    let arch = env::consts::ARCH;

    let (os_name, ext) = match os {
        "macos" => ("darwin", "tar.gz"),
        "linux" => ("linux", "tar.gz"),
        "windows" => ("windows", "zip"),
        other => {
            eprintln!("Unsupported OS: {}", other);
            process::exit(1);
        }
    };

    let arch = match arch {
        "aarch64" => "arm64",
        _ => "x86_64",
    };

    let update_url = format!(
        "https://github.com/StanleyMasinde/twitter/releases/latest/download/{}-{}-{}.{}",
        binary_name, os_name, arch, ext
    );

    let archive_name = format!("{}.{}", binary_name, ext);

    let mut stream = match reqwest::get(update_url).await {
        Ok(res) => res.bytes_stream(),
        Err(err) => {
            eprintln!("Could not make a network req: {}", err);
            process::exit(1)
        }
    };

    let mut new_file = match File::create(&archive_name).await {
        Ok(file) => file,
        Err(err) => {
            eprintln!("Failed to create temp file: {}", err);
            process::exit(1)
        }
    };

    while let Some(item) = stream.next().await {
        let bytes = match item {
            Ok(b) => b,
            Err(err) => {
                eprintln!("Failed to read downloaded file: {err}");
                process::exit(1)
            }
        };

        let _ = new_file.write_all(&bytes).await;
    }

    let extract_status = match os {
        "windows" => Command::new("powershell")
            .args([
                "-Command",
                &format!(
                    "Expand-Archive -Path {} -DestinationPath . -Force",
                    archive_name
                ),
            ])
            .status(),
        _ => Command::new("tar").args(["-xzf", &archive_name]).status(),
    };

    if extract_status.is_err() {
        eprintln!("Failed to extract the update file.");
        process::exit(1)
    }

    let install_status = Command::new("sudo")
        .arg("install")
        .args(["-sm", "755", binary_name, "/usr/local/bin/"])
        .status();

    if install_status.is_err() {
        eprintln!("Failed to update");
        process::exit(1)
    }

    let output = match Command::new("twitter").arg("--version").output() {
        Ok(output) => output,
        Err(err) => {
            eprintln!("Could not get the new version name: {}", err);
            process::exit(1)
        }
    };

    let stdout = output.stdout;
    let output_from_string = String::from_utf8_lossy(&stdout).trim().to_string();
    println!("Updated to version: {}", output_from_string)
}
