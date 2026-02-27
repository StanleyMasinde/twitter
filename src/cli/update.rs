use indicatif::{ProgressBar, ProgressStyle};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::{
    cell::RefCell,
    env,
    io::{self, Read, Write},
    path::{Path, PathBuf},
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};

use crate::utils::gracefully_exit;

const REPO: &str = "StanleyMasinde/twitter";

enum InstallOutcome {
    Immediate,
    #[cfg(windows)]
    Deferred,
}

pub fn run() {
    let os = env::consts::OS;
    let arch = env::consts::ARCH;
    let temp_dir = env::temp_dir();

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

    if let Err(err) = std::fs::create_dir_all(&work_dir) {
        let message = format!("Failed to create temp dir: {err}");
        gracefully_exit(&message)
    }

    let release_json = match fetch_release_json("latest") {
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
    if let Err(err) = download_with_progress(&download_url, &archive_path) {
        let message = format!("Failed to download update: {err}");
        gracefully_exit(&message)
    }

    if let Some(expected_sha) = digest {
        println!("> Verifying checksum");
        match compute_sha256(&archive_path) {
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
    if let Err(err) = std::fs::create_dir_all(&extract_dir) {
        let message = format!("Failed to create extract dir: {err}");
        gracefully_exit(&message)
    }

    let extracted = match extract_binary(&archive_path, ext, &extract_dir, binary_name) {
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
    let _ = std::fs::remove_dir_all(&work_dir);

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

fn fetch_release_json(version: &str) -> Result<Value, String> {
    let url = if version == "latest" {
        format!("https://api.github.com/repos/{}/releases/latest", REPO)
    } else {
        format!(
            "https://api.github.com/repos/{}/releases/tags/{}",
            REPO, version
        )
    };

    let response = curl_rest::Client::default()
        .get()
        .header(curl_rest::Header::UserAgent("twitter-cli".into()))
        .send(&url)
        .map_err(|err| err.to_string())?;

    if (200..300).contains(&response.status.as_u16()) {
        serde_json::from_slice::<Value>(&response.body).map_err(|err| err.to_string())
    } else {
        Err(format!(
            "HTTP {}: {}",
            response.status,
            String::from_utf8_lossy(&response.body)
        ))
    }
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

fn download_with_progress(url: &str, dest: &Path) -> Result<(), String> {
    use curl::easy::{Easy, WriteError};

    let mut easy = Easy::new();
    easy.url(url).map_err(|err| err.to_string())?;
    easy.follow_location(true).map_err(|err| err.to_string())?;
    easy.useragent("twitter-cli")
        .map_err(|err| err.to_string())?;

    let file = std::fs::File::create(dest).map_err(|err| err.to_string())?;
    let file = RefCell::new(file);

    let pb = ProgressBar::new(0);
    pb.set_style(
        ProgressStyle::default_bar()
            .template(
                "{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes}",
            )
            .unwrap()
            .progress_chars("##-"),
    );

    {
        let mut transfer = easy.transfer();
        let progress_from_headers = pb.clone();
        transfer
            .header_function(move |header| {
                if let Ok(line) = std::str::from_utf8(header)
                    && let Some(value) = line.to_ascii_lowercase().strip_prefix("content-length:")
                    && let Ok(total_size) = value.trim().parse::<u64>()
                {
                    progress_from_headers.set_length(total_size);
                }
                true
            })
            .map_err(|err| err.to_string())?;

        let progress_from_body = pb.clone();
        transfer
            .write_function(move |data| {
                file.borrow_mut()
                    .write_all(data)
                    .map_err(|_| WriteError::Pause)?;
                progress_from_body.inc(data.len() as u64);
                Ok(data.len())
            })
            .map_err(|err| err.to_string())?;

        transfer.perform().map_err(|err| err.to_string())?;
    }

    let status = easy.response_code().map_err(|err| err.to_string())?;
    if !(200..300).contains(&status) {
        let _ = std::fs::remove_file(dest);
        return Err(format!("HTTP {status} while downloading update"));
    }

    pb.finish_with_message("> Download complete");

    Ok(())
}

fn compute_sha256(path: &Path) -> io::Result<String> {
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
}

fn extract_binary(
    archive_path: &Path,
    ext: &str,
    extract_dir: &Path,
    binary_name: &str,
) -> io::Result<PathBuf> {
    match ext {
        "tar.gz" => extract_from_tar_gz(archive_path, extract_dir, binary_name),
        "zip" => extract_from_zip(archive_path, extract_dir, binary_name),
        other => Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("Unsupported archive type: {}", other),
        )),
    }
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
        Ok(InstallOutcome::Immediate)
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
