use std::fs;
use std::path::Path;
use std::process::Command;
use std::thread;
use std::time::Duration;

const CONFIG_FILE: &str = "/etc/xos/updates.conf";
const FALLBACK_CONFIG: &str = "/tmp/xos_updates.conf";
const STATE_FILE: &str = "/run/xos/updates.state";
const FALLBACK_STATE: &str = "/tmp/xos_updates.state";
const TRIGGER_FILE: &str = "/run/xos/update.trigger";
const FALLBACK_TRIGGER: &str = "/tmp/xos_update.trigger";
const LOG_FILE: &str = "/var/log/xos/update-manager.log";

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
    log_message("XOS Update Manager starting...");

    let use_run_dir = check_run_dir();
    let state_path = if use_run_dir {
        STATE_FILE
    } else {
        FALLBACK_STATE
    };

    let trigger_path = if use_run_dir {
        TRIGGER_FILE
    } else {
        FALLBACK_TRIGGER
    };

    // Initial state
    write_state(state_path, "Idle", 3, "Never");

    loop {
        // 1. Check if update was triggered
        if Path::new(trigger_path).exists() {
            let trigger_content = fs::read_to_string(trigger_path).unwrap_or_default();
            if trigger_content.trim() == "upgrade" {
                match fs::remove_file(trigger_path) {
                    Ok(_) => {
                        run_upgrade(state_path);
                    }
                    Err(e) => {
                        log_message(&format!("ERROR: Failed to remove trigger file {}: {}. Aborting upgrade to prevent infinite loop.", trigger_path, e));
                    }
                }
            }
        }

        // 2. Mock checking updates periodic check (every 1 hour in reality, let's do small mock checks)
        // We'll read from config to check settings
        let config_path = if Path::new(CONFIG_FILE).exists() {
            CONFIG_FILE
        } else {
            FALLBACK_CONFIG
        };

        if !Path::new(config_path).exists() {
            let _ = fs::write(config_path, "auto_download=false\nschedule=daily\n");
        }

        thread::sleep(Duration::from_secs(4));
    }
}

fn write_state(path: &str, status: &str, count: u32, last_check: &str) {
    let content = format!(
        "status={}\nupdates_available={}\nlast_check={}\n",
        status, count, last_check
    );
    let _ = atomic_write(path, &content);
}

fn run_upgrade(state_path: &str) {
    let now_str = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
    log_message("Upgrade triggered. Initiating system upgrade process.");

    // Step 1: Btrfs Snapshot
    write_state(state_path, "Snapshotting", 3, &now_str);
    create_btrfs_snapshot();

    // Step 2: Run Upgrade
    write_state(state_path, "Upgrading", 3, &now_str);
    
    // Simulate upgrade delay
    log_message("Running pacman system upgrade (simulated)...");
    thread::sleep(Duration::from_secs(5));

    log_message("System upgrade completed successfully.");
    write_state(state_path, "Idle", 0, &now_str);
}

fn create_btrfs_snapshot() {
    log_message("Creating pre-update Btrfs snapshot...");

    // Check if root is Btrfs and btrfs tool exists
    let output = Command::new("btrfs")
        .args(["subvolume", "show", "/"])
        .output();

    if let Ok(out) = output {
        if out.status.success() {
            // Root is btrfs! Let's snapshot `@` to `@snapshots`
            let snapshot_name = format!("/.snapshots/pre-update-{}", chrono::Local::now().format("%Y%m%d%H%M%S"));
            let snapshot_result = Command::new("btrfs")
                .args(["subvolume", "snapshot", "/", &snapshot_name])
                .output();

            match snapshot_result {
                Ok(r) if r.status.success() => {
                    log_message(&format!("Btrfs snapshot created successfully at {}.", snapshot_name));
                }
                _ => {
                    log_message("WARNING: Failed to create Btrfs snapshot. Continuing with update anyway.");
                }
            }
            return;
        }
    }

    log_message("Btrfs subvolume not detected or btrfs command not found. Skipping snapshot (simulated snapshot recorded).");
}
