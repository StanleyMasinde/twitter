use futures_util::StreamExt;
use indicatif::{ProgressBar, ProgressStyle};
use std::{
    env,
    process::{self, Command},
};
use tokio::{
    fs::{self, File},
    io::AsyncWriteExt,
};

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
        other => {
            eprintln!("Unsupported OS: {}", other);
            process::exit(1);
        }
    };

    let update_url = format!(
        "https://github.com/StanleyMasinde/twitter/releases/latest/download/{}-{}-{}.{}",
        binary_name, os_name, arch, ext
    );

    let archive_name = temp_dir.join(format!("{}.{}", binary_name, ext));

    let head = match client.head(&update_url).send().await {
        Ok(res) => res,
        Err(_) => process::exit(1),
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

    while let Some(chunk) = stream.next().await {
        let bytes = match chunk {
            Ok(b) => b,
            Err(err) => {
                eprintln!("Failed to read downloaded file: {err}");
                process::exit(1)
            }
        };

        pb.inc(bytes.len() as u64);

        let _ = new_file.write_all(&bytes).await;
    }

    let extract_status = match os {
        "windows" => Command::new("powershell")
            .args([
                "-Command",
                &format!(
                    "Expand-Archive -Path {} -DestinationPath . -Force",
                    archive_name.display()
                ),
            ])
            .status(),
        _ => Command::new("tar")
            .args(["-xzf", &archive_name.display().to_string()])
            .status(),
    };
    pb.finish_with_message("> Download complete");

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

    println!("> Cleaning up");
    let _ = fs::remove_file(archive_name).await;
    let _ = fs::remove_file(binary_name).await;

    let stdout = output.stdout;
    let output_from_string = String::from_utf8_lossy(&stdout).trim().to_string();
    println!("> Updated to version: {}", output_from_string)
}
