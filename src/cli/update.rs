use futures_util::StreamExt;
use std::process::{self, Command};
use tokio::{fs::File, io::AsyncWriteExt};

pub async fn run() {
    let update_url = "https://github.com/StanleyMasinde/twitter/releases/latest/download/twitter-darwin-arm64.tar.gz";
    let archive_name = "twitter.tar.gz";
    let binary_name = "twitter";

    let mut stream = match reqwest::get(update_url).await {
        Ok(res) => res.bytes_stream(),
        Err(err) => {
            eprintln!("Could not make a network req: {}", err);
            process::exit(1)
        }
    };

    let mut new_file = match File::create(archive_name).await {
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

    let extract_status = Command::new("tar").args(["-xzf", archive_name]).status();
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
    println!("Updated to version: v{}", output_from_string)
}
