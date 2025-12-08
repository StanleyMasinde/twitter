use futures_util::StreamExt;
use indicatif::{ProgressBar, ProgressStyle};
use std::{
    env,
    process::Command,
};
use tokio::{
    fs::{self, File},
    io::AsyncWriteExt,
};

use crate::utils::gracefully_exit;

pub async fn run() {
    let binary_name = "twitter";
    let os = env::consts::OS;
    let arch = env::consts::ARCH;
    let temp_dir = env::temp_dir();
    let client = reqwest::Client::new();

    let (os_name, ext) = match os {
        "macos" => ("darwin", "tar.gz"),
        "linux" => ("linux", "tar.gz"),
        "windows" => ("windows", "zip"),
        other_os => {
            let message = format!("Sorry, self update for {} is not supported yet.", other_os);
            gracefully_exit(&message)
        }
    };

    let update_url = format!(
        "https://github.com/StanleyMasinde/twitter/releases/latest/download/{}-{}-{}.{}",
        binary_name, os_name, arch, ext
    );

    let archive_name = temp_dir.join(format!("{}.{}", binary_name, ext));

    let head = match client.head(&update_url).send().await {
        Ok(res) => res,
        Err(_) => gracefully_exit("Failed to to fetch update information."),
    };

    let total_size = head
        .headers()
        .get(reqwest::header::CONTENT_LENGTH)
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.parse::<u64>().ok())
        .unwrap_or(0);

    let pb = ProgressBar::new(total_size);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta})")
            .unwrap()
            .progress_chars("##-"),
    );

    let mut stream = match reqwest::get(update_url).await {
        Ok(res) => res.bytes_stream(),
        Err(err) => {
            let message = format!("Could not make a network req: {}", err);
            gracefully_exit(&message)
        }
    };

    let mut new_file = match File::create(&archive_name).await {
        Ok(file) => file,
        Err(err) => {
            let message = format!("Failed to create temp file: {}", err);
            gracefully_exit(&message)
        }
    };

    while let Some(chunk) = stream.next().await {
        let bytes = match chunk {
            Ok(b) => b,
            Err(err) => {
                let message = format!("Failed to read downloaded file: {err}");
                gracefully_exit(&message)
            }
        };

        pb.inc(bytes.len() as u64);

        let _ = new_file.write_all(&bytes).await;
    }

    let extract_status = match os {
        "windows" => {
            let fname = archive_name.to_str().unwrap();
            Command::new("tar").args(["-xf", fname]).status()
        }
        _ => {
            let fname = archive_name.to_str().unwrap();
            Command::new("tar").args(["-xzf", fname]).status()
        }
    };
    pb.finish_with_message("> Download complete");

    if extract_status.is_err() {
        let message = "Failed to extract the update file.".to_string();
        gracefully_exit(&message)
    }

    #[cfg(unix)]
    let install_status = Command::new("sudo")
        .arg("install")
        .args(["-sm", "755", binary_name, "/usr/local/bin/"])
        .status();

    #[cfg(windows)]
    let install_status = Command::new("powershell")
        .args([
            "-NoProfile",
            "-Command",
            r#"$dest = \"$env:USERPROFILE\bin\"; 
           New-Item -ItemType Directory -Force -Path $dest | Out-Null; 
           Move-Item -Force \"./twitter.exe\" \"$dest\twitter.exe\";"#,
        ])
        .status();

    if install_status.is_err() {
        let message = "Failed to update".to_string();
        gracefully_exit(&message)
    }

    let output = match Command::new("twitter").arg("--version").output() {
        Ok(output) => output,
        Err(err) => {
            let message = format!("Could not get the new version name: {}", err);
            gracefully_exit(&message)
        }
    };

    println!("> Cleaning up");
    let _ = fs::remove_file(archive_name).await;
    let _ = fs::remove_file(binary_name).await;

    let stdout = output.stdout;
    let output_from_string = String::from_utf8_lossy(&stdout).trim().to_string();
    println!("> Updated to version: {}", output_from_string)
}
