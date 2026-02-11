use futures_util::StreamExt;
use indicatif::{ProgressBar, ProgressStyle};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::{
    env,
    io::{self, Read, Write},
    path::{Path, PathBuf},
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};
use tokio::{
    fs::{self, File},
    io::AsyncWriteExt,
    task,
};

use crate::utils::gracefully_exit;

const REPO: &str = "StanleyMasinde/twitter";

enum InstallOutcome {
    Immediate,
    #[cfg(windows)]
    Deferred,
}

pub async fn run() {
    let os = env::consts::OS;
    let arch = env::consts::ARCH;
    let temp_dir = env::temp_dir();
    let client = reqwest::Client::new();

    let os_name = match os {
        "macos" => "darwin",
        "linux" => "linux",
        "windows" => "windows",
        other_os => {
            let message = format!("Sorry, self update for {} is not supported yet.", other_os);
            gracefully_exit(&message)
        }
    };

    let arch_name = match normalize_arch(arch) {
        Some(arch_name) => arch_name,
        None => {
            let message = format!("Sorry, self update for {} is not supported yet.", arch);
            gracefully_exit(&message)
        }
    };

    let ext = if os_name == "windows" {
        "zip"
    } else {
        "tar.gz"
    };
    let binary_name = if os_name == "windows" {
        "twitter.exe"
    } else {
        "twitter"
    };

    let filename = format!("twitter-{}-{}.{}", os_name, arch_name, ext);
    let work_dir = temp_dir.join(format!("twitter-update-{}", unique_suffix()));

    if let Err(err) = fs::create_dir_all(&work_dir).await {
        let message = format!("Failed to create temp dir: {err}");
        gracefully_exit(&message)
    }

    let release_json = match fetch_release_json(&client, "latest").await {
        Ok(json) => json,
        Err(err) => gracefully_exit(&format!("Failed to fetch release data: {err}")),
    };

    let (download_url, digest, _available) = match find_asset(&release_json, &filename) {
        Ok(found) => found,
        Err((message, available)) => {
            let mut full = message;
            if !available.is_empty() {
                full.push_str("\nAvailable assets:");
                for asset in available {
                    full.push_str(&format!("\n  - {}", asset));
                }
            }
            gracefully_exit(&full)
        }
    };

    println!("> Downloading {}", filename);
    let archive_path = work_dir.join(&filename);
    if let Err(err) = download_with_progress(&client, &download_url, &archive_path).await {
        let message = format!("Failed to download update: {err}");
        gracefully_exit(&message)
    }

    if let Some(expected_sha) = digest {
        println!("> Verifying checksum");
        match compute_sha256(&archive_path).await {
            Ok(actual) if actual == expected_sha => {}
            Ok(actual) => {
                let message = format!(
                    "Checksum verification failed.\nExpected: {}\nGot: {}",
                    expected_sha, actual
                );
                gracefully_exit(&message)
            }
            Err(err) => {
                let message = format!("Failed to compute checksum: {err}");
                gracefully_exit(&message)
            }
        }
    } else {
        println!("> No checksum available for this release (skipping verification)");
    }

    println!("> Extracting");
    let extract_dir = work_dir.join("extract");
    if let Err(err) = fs::create_dir_all(&extract_dir).await {
        let message = format!("Failed to create extract dir: {err}");
        gracefully_exit(&message)
    }

    let extracted = match extract_binary(&archive_path, ext, &extract_dir, binary_name).await {
        Ok(path) => path,
        Err(err) => {
            let message = format!("Failed to extract update file: {err}");
            gracefully_exit(&message)
        }
    };

    let target_path = resolve_install_path(binary_name);
    let outcome = match install_binary(&extracted, &target_path) {
        Ok(outcome) => outcome,
        Err(err) => {
            let message = format!(
                "Failed to update.\nTarget: {}\nReason: {}",
                target_path.display(),
                err
            );
            gracefully_exit(&message)
        }
    };

    println!("> Cleaning up");
    let _ = fs::remove_dir_all(&work_dir).await;

    match outcome {
        InstallOutcome::Immediate => {
            let version = match Command::new(&target_path).arg("--version").output() {
                Ok(output) => {
                    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
                    if stdout.is_empty() {
                        "unknown".to_string()
                    } else {
                        stdout
                    }
                }
                Err(err) => {
                    let message = format!("Could not get the new version name: {}", err);
                    gracefully_exit(&message)
                }
            };
            println!("> Updated to {}", version);
        }
        #[cfg(windows)]
        InstallOutcome::Deferred => {
            println!("> Update scheduled for completion after this process exits");
            println!("> Run `twitter --version` in a new terminal to verify");
        }
    }
}

fn normalize_arch(arch: &str) -> Option<&'static str> {
    match arch {
        "x86_64" | "amd64" => Some("x86_64"),
        "aarch64" | "arm64" => Some("aarch64"),
        "armv7l" | "armv6l" | "arm" => Some("arm"),
        _ => None,
    }
}

fn unique_suffix() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0)
}

async fn fetch_release_json(
    client: &reqwest::Client,
    version: &str,
) -> Result<Value, reqwest::Error> {
    let url = if version == "latest" {
        format!("https://api.github.com/repos/{}/releases/latest", REPO)
    } else {
        format!(
            "https://api.github.com/repos/{}/releases/tags/{}",
            REPO, version
        )
    };

    client
        .get(url)
        .header(reqwest::header::USER_AGENT, "twitter-cli")
        .send()
        .await?
        .error_for_status()?
        .json::<Value>()
        .await
}

type FindAssetOk = (String, Option<String>, Vec<String>);
type FindAssetErr = (String, Vec<String>);

fn find_asset(release_json: &Value, filename: &str) -> Result<FindAssetOk, FindAssetErr> {
    let assets = release_json
        .get("assets")
        .and_then(|v| v.as_array())
        .ok_or_else(|| ("Release data missing assets".to_string(), Vec::new()))?;

    let mut available = Vec::new();
    for asset in assets {
        if let Some(name) = asset.get("name").and_then(|v| v.as_str()) {
            available.push(name.to_string());
            if name == filename {
                let url = asset
                    .get("browser_download_url")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| ("Asset missing download URL".to_string(), available.clone()))?
                    .to_string();

                let digest = asset
                    .get("digest")
                    .and_then(|v| v.as_str())
                    .and_then(|d| d.strip_prefix("sha256:").map(|s| s.to_string()));

                return Ok((url, digest, available));
            }
        }
    }

    Err((format!("Could not find asset '{}'", filename), available))
}

async fn download_with_progress(
    client: &reqwest::Client,
    url: &str,
    dest: &Path,
) -> Result<(), String> {
    let response = client
        .get(url)
        .send()
        .await
        .map_err(|err| err.to_string())?
        .error_for_status()
        .map_err(|err| err.to_string())?;

    let total_size = response.content_length().unwrap_or(0);

    let pb = ProgressBar::new(total_size);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta})")
            .unwrap()
            .progress_chars("##-"),
    );

    let mut stream = response.bytes_stream();
    let mut file = File::create(dest).await.map_err(|err| err.to_string())?;

    while let Some(chunk) = stream.next().await {
        let bytes = chunk.map_err(|err| err.to_string())?;
        file.write_all(&bytes)
            .await
            .map_err(|err| err.to_string())?;
        pb.inc(bytes.len() as u64);
    }

    pb.finish_with_message("> Download complete");
    Ok(())
}

async fn compute_sha256(path: &Path) -> io::Result<String> {
    let path = path.to_path_buf();
    task::spawn_blocking(move || {
        let mut file = std::fs::File::open(path)?;
        let mut hasher = Sha256::new();
        let mut buf = [0u8; 8192];
        loop {
            let read = file.read(&mut buf)?;
            if read == 0 {
                break;
            }
            hasher.update(&buf[..read]);
        }
        Ok(hex::encode(hasher.finalize()))
    })
    .await
    .unwrap_or_else(|err| Err(io::Error::other(err)))
}

async fn extract_binary(
    archive_path: &Path,
    ext: &str,
    extract_dir: &Path,
    binary_name: &str,
) -> io::Result<PathBuf> {
    let archive_path = archive_path.to_path_buf();
    let extract_dir = extract_dir.to_path_buf();
    let binary_name = binary_name.to_string();
    let ext = ext.to_string();

    task::spawn_blocking(move || match ext.as_str() {
        "tar.gz" => extract_from_tar_gz(&archive_path, &extract_dir, &binary_name),
        "zip" => extract_from_zip(&archive_path, &extract_dir, &binary_name),
        other => Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("Unsupported archive type: {}", other),
        )),
    })
    .await
    .unwrap_or_else(|err| Err(io::Error::other(err)))
}

fn extract_from_tar_gz(
    archive_path: &Path,
    extract_dir: &Path,
    binary_name: &str,
) -> io::Result<PathBuf> {
    let file = std::fs::File::open(archive_path)?;
    let decoder = flate2::read::GzDecoder::new(file);
    let mut archive = tar::Archive::new(decoder);

    for entry in archive.entries()? {
        let mut entry = entry?;
        let path = entry.path()?;
        if path.file_name().and_then(|p| p.to_str()) == Some(binary_name) {
            let dest = extract_dir.join(binary_name);
            entry.unpack(&dest)?;
            return Ok(dest);
        }
    }

    Err(io::Error::new(
        io::ErrorKind::NotFound,
        "Binary not found in archive",
    ))
}

fn extract_from_zip(
    archive_path: &Path,
    extract_dir: &Path,
    binary_name: &str,
) -> io::Result<PathBuf> {
    let file = std::fs::File::open(archive_path)?;
    let mut archive = zip::ZipArchive::new(file)?;

    for i in 0..archive.len() {
        let mut entry = archive.by_index(i)?;
        let name = match entry.enclosed_name() {
            Some(name) => name,
            None => continue,
        };

        if name.file_name().and_then(|p| p.to_str()) == Some(binary_name) {
            let dest = extract_dir.join(binary_name);
            let mut output = std::fs::File::create(&dest)?;
            io::copy(&mut entry, &mut output)?;
            return Ok(dest);
        }
    }

    Err(io::Error::new(
        io::ErrorKind::NotFound,
        "Binary not found in archive",
    ))
}

fn resolve_install_path(binary_name: &str) -> PathBuf {
    if let Ok(dir) = env::var("TWITTER_INSTALL") {
        return PathBuf::from(dir).join(binary_name);
    }

    if let Ok(exe) = env::current_exe()
        && exe.file_name().and_then(|p| p.to_str()) == Some(binary_name)
    {
        return exe;
    }

    PathBuf::from("/usr/local/bin").join(binary_name)
}

fn install_binary(source: &Path, target: &Path) -> io::Result<InstallOutcome> {
    let target_dir = target
        .parent()
        .ok_or_else(|| io::Error::other("Invalid target path"))?;

    if !target_dir.exists() {
        std::fs::create_dir_all(target_dir)?;
    }

    let temp_target = target_dir.join(format!(
        ".{}.new",
        target
            .file_name()
            .and_then(|p| p.to_str())
            .unwrap_or("twitter")
    ));

    let mut input = std::fs::File::open(source)?;
    let mut output = std::fs::File::create(&temp_target)?;
    io::copy(&mut input, &mut output)?;
    output.flush()?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = output.metadata()?.permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(&temp_target, perms)?;
    }

    drop(output);

    #[cfg(windows)]
    {
        schedule_windows_replace(&temp_target, target)?;
        return Ok(InstallOutcome::Deferred);
    }

    #[cfg(not(windows))]
    {
        std::fs::rename(&temp_target, target)?;
        return Ok(InstallOutcome::Immediate);
    }
}

#[cfg(windows)]
fn schedule_windows_replace(source: &Path, target: &Path) -> io::Result<()> {
    let source = ps_single_quoted(source);
    let target = ps_single_quoted(target);
    let script = format!(
        "$src='{source}';$dst='{target}';for($i=0;$i -lt 120;$i++){{try{{Move-Item -LiteralPath $src -Destination $dst -Force;exit 0}}catch{{Start-Sleep -Milliseconds 250}}}};exit 1"
    );

    Command::new("powershell")
        .args([
            "-NoProfile",
            "-NonInteractive",
            "-ExecutionPolicy",
            "Bypass",
            "-WindowStyle",
            "Hidden",
            "-Command",
            &script,
        ])
        .spawn()
        .map(|_| ())
}

#[cfg(windows)]
fn ps_single_quoted(path: &Path) -> String {
    path.to_string_lossy().replace('\'', "''")
}
