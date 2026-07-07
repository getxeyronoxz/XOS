use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::thread;
use std::time::Duration;
use serde::{Deserialize, Serialize};

const CONFIG_FILE: &str = "/etc/xos/permissions.toml";
const FALLBACK_CONFIG: &str = "/tmp/xos_permissions.toml";
const STATE_FILE: &str = "/run/xos/permissions.state";
const FALLBACK_STATE: &str = "/tmp/xos_permissions.state";
const LOG_FILE: &str = "/var/log/xos/permission-daemon.log";

#[derive(Serialize, Deserialize, Clone, Debug)]
struct AppPermissions {
    camera: bool,
    microphone: bool,
    location: bool,
    network: bool,
}

#[derive(Serialize, Deserialize, Default, Clone, Debug)]
struct PermissionsConfig {
    apps: HashMap<String, AppPermissions>,
}

fn log_message(msg: &str) {
    let now = chrono::Local::now();
    let log_msg = format!("[{}] {}\n", now.format("%Y-%m-%d %H:%M:%S"), msg);

    if let Some(parent) = Path::new(LOG_FILE).parent() {
        let _ = fs::create_dir_all(parent);
    }

    if let Ok(mut file) = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(LOG_FILE)
    {
        use std::io::Write;
        let _ = file.write_all(log_msg.as_bytes());
    } else {
        eprintln!("Log fallback: {msg}");
    }
}

struct LogCacheEntry {
    perms: AppPermissions,
    last_logged: std::time::Instant,
}

fn check_run_dir() -> bool {
    let run_dir = Path::new("/run/xos");
    if fs::create_dir_all(run_dir).is_err() {
        return false;
    }
    let test_file = run_dir.join(".xos_write_test");
    match fs::write(&test_file, b"") {
        Ok(_) => {
            let _ = fs::remove_file(test_file);
            true
        }
        Err(_) => false,
    }
}

fn atomic_write<P: AsRef<Path>>(path: P, content: &str) -> std::io::Result<()> {
    let path = path.as_ref();
    let parent = path.parent().ok_or_else(|| {
        std::io::Error::new(std::io::ErrorKind::InvalidInput, "Path has no parent directory")
    })?;
    let file_name = path.file_name().ok_or_else(|| {
        std::io::Error::new(std::io::ErrorKind::InvalidInput, "Path has no file name")
    })?;
    let mut temp_name = file_name.to_os_string();
    temp_name.push(".tmp");
    let temp_path = parent.join(temp_name);
    
    fs::write(&temp_path, content)?;
    if let Err(e) = fs::rename(&temp_path, path) {
        let _ = fs::remove_file(&temp_path);
        return Err(e);
    }
    Ok(())
}

fn main() {
    log_message("XOS Permission Daemon starting...");

    let mut log_cache: HashMap<String, LogCacheEntry> = HashMap::new();

    loop {
        // 1. Read / Initialize Config
        let config_path = if Path::new(CONFIG_FILE).exists() {
            CONFIG_FILE
        } else {
            FALLBACK_CONFIG
        };

        let config = match read_config(config_path) {
            Ok(cfg) => cfg,
            Err(_) => {
                let default_cfg = get_default_config();
                let _ = write_config(config_path, &default_cfg);
                default_cfg
            }
        };

        // 2. Write state file for UI
        let state_path = if check_run_dir() {
            STATE_FILE
        } else {
            FALLBACK_STATE
        };

        let mut state_content = String::new();
        for (app, perms) in &config.apps {
            state_content.push_str(&format!("{}_camera={}\n", app, perms.camera));
            state_content.push_str(&format!("{}_microphone={}\n", app, perms.microphone));
            state_content.push_str(&format!("{}_location={}\n", app, perms.location));
            state_content.push_str(&format!("{}_network={}\n", app, perms.network));
            
            // 3. Enforce permissions
            enforce_permissions(app, perms, &mut log_cache);
        }

        if let Err(e) = atomic_write(state_path, &state_content) {
            log_message(&format!("ERROR: Failed to write permissions state: {e}"));
        }

        thread::sleep(Duration::from_secs(4));
    }
}

fn enforce_permissions(app: &str, perms: &AppPermissions, log_cache: &mut HashMap<String, LogCacheEntry>) {
    // In real XOS, this would block mic/cam access using wireplumber/pipewire APIs,
    // or modify iptables/nftables rules to block network access for uid/process.
    // Here we log the current enforcement state.
    let now = std::time::Instant::now();
    let should_log = match log_cache.get(app) {
        Some(entry) => {
            let perms_changed = entry.perms.camera != perms.camera
                || entry.perms.microphone != perms.microphone
                || entry.perms.location != perms.location
                || entry.perms.network != perms.network;
            let time_elapsed = now.duration_since(entry.last_logged) >= Duration::from_secs(300);
            perms_changed || time_elapsed
        }
        None => true,
    };

    if should_log {
        log_message(&format!(
            "Enforcing permissions for {}: Camera={}, Mic={}, Location={}, Net={}",
            app, perms.camera, perms.microphone, perms.location, perms.network
        ));
        log_cache.insert(app.to_string(), LogCacheEntry {
            perms: perms.clone(),
            last_logged: now,
        });
    }
}

fn read_config(path: &str) -> Result<PermissionsConfig, Box<dyn std::error::Error>> {
    let content = fs::read_to_string(path)?;
    let config: PermissionsConfig = toml::from_str(&content)?;
    Ok(config)
}

fn write_config(path: &str, config: &PermissionsConfig) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(parent) = Path::new(path).parent() {
        fs::create_dir_all(parent)?;
    }
    let content = toml::to_string(config)?;
    fs::write(path, content)?;
    Ok(())
}

fn get_default_config() -> PermissionsConfig {
    let mut apps = HashMap::new();
    apps.insert(
        "firefox".to_string(),
        AppPermissions {
            camera: true,
            microphone: true,
            location: false,
            network: true,
        },
    );
    apps.insert(
        "foot".to_string(),
        AppPermissions {
            camera: false,
            microphone: false,
            location: false,
            network: false,
        },
    );
    apps.insert(
        "xos-file-manager".to_string(),
        AppPermissions {
            camera: false,
            microphone: false,
            location: false,
            network: true,
        },
    );
    PermissionsConfig { apps }
}
