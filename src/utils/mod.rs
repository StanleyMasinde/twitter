use std::{
    env, fs,
    os::unix::fs::PermissionsExt,
    path::PathBuf,
    process::{Command, ExitStatus},
};

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
    let editor = env::var("EDITOR")
        .or_else(|_| env::var("VISUAL"))
        .unwrap_or_else(|_| "vi".to_string());

    Command::new(editor)
        .arg(file)
        .status()
        .expect("Failed to open the editor.")
}

pub fn check_permissions(path: &PathBuf, is_dir: bool) {
    if let Ok(metadata) = fs::metadata(path) {
        let mode = metadata.permissions().mode() & 0o777;

        let expected = if is_dir { 0o700 } else { 0o600 };

        if mode != expected {
            eprintln!(
                "⚠️  Permissions for {:?} are {:o}, expected {:o}",
                path, mode, expected
            );
            eprintln!("Run chmod {:o} {:?}", expected, path)
        }
    }
}
