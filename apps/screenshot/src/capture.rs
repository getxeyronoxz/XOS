use std::path::PathBuf;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CaptureMode {
    Region,
    FullScreen,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CaptureTarget {
    Clipboard,
    Save,
}

pub fn capture(mode: CaptureMode, target: CaptureTarget) -> Result<(), String> {
    match target {
        CaptureTarget::Clipboard => capture_to_clipboard(mode),
        CaptureTarget::Save => {
            let path = default_save_path()?;
            capture_to_file(mode, &path)?;
            eprintln!("Saved screenshot: {}", path.display());
            Ok(())
        }
    }
}

fn capture_to_clipboard(mode: CaptureMode) -> Result<(), String> {
    let temp = std::env::temp_dir().join(format!(
        "xos-screenshot-{}.png",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_millis())
            .unwrap_or(0)
    ));

    capture_to_file(mode, &temp)?;
    copy_file_to_clipboard(&temp)?;
    let _ = std::fs::remove_file(&temp);
    Ok(())
}

fn capture_to_file(mode: CaptureMode, path: &PathBuf) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|err| format!("Failed to create directory {}: {err}", parent.display()))?;
    }

    match mode {
        CaptureMode::FullScreen => run_grim(&[], path),
        CaptureMode::Region => {
            let geometry = select_region()?;
            run_grim(&["-g", &geometry], path)
        }
    }
}

fn select_region() -> Result<String, String> {
    let output = Command::new("slurp")
        .output()
        .map_err(|err| format!("Failed to run slurp: {err}"))?;

    if !output.status.success() {
        return Err("Region selection cancelled".to_string());
    }

    let geometry = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if geometry.is_empty() {
        return Err("No region selected".to_string());
    }

    Ok(geometry)
}

fn run_grim(args: &[&str], path: &PathBuf) -> Result<(), String> {
    let mut command = Command::new("grim");
    command.args(args).arg(path);

    let status = command
        .status()
        .map_err(|err| format!("Failed to run grim: {err}"))?;

    if status.success() {
        Ok(())
    } else {
        Err("grim exited with an error".to_string())
    }
}

fn copy_file_to_clipboard(path: &PathBuf) -> Result<(), String> {
    let status = Command::new("wl-copy")
        .args(["--type", "image/png"])
        .arg(path)
        .status()
        .map_err(|err| format!("Failed to run wl-copy: {err}"))?;

    if status.success() {
        Ok(())
    } else {
        Err("wl-copy exited with an error".to_string())
    }
}

fn default_save_path() -> Result<PathBuf, String> {
    let home = std::env::var("HOME").map_err(|_| "HOME is not set".to_string())?;
    let pictures = PathBuf::from(home).join("Pictures");
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    Ok(pictures.join(format!("screenshot-{timestamp}.png")))
}
