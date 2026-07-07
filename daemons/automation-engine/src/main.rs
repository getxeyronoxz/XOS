use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::process::Command;
use std::thread;
use std::time::{Duration, Instant};
use serde::{Deserialize, Serialize};

const CONFIG_FILE: &str = "/etc/xos/automations.toml";
const FALLBACK_CONFIG: &str = "/tmp/xos_automations.toml";
const BATTERY_STATE: &str = "/run/xos/battery.state";
const FALLBACK_BATTERY: &str = "/tmp/xos_battery.state";
const THERMAL_STATE: &str = "/run/xos/thermal.state";
const FALLBACK_THERMAL: &str = "/tmp/xos_thermal.state";
const LOG_FILE: &str = "/var/log/xos/automation-engine.log";

#[derive(Serialize, Deserialize, Clone, Debug)]
struct Rule {
    name: String,
    trigger: String, // e.g. "battery_pct < 20", "cpu_temp > 85", "power_status == Discharging"
    action: String,  // command to run
}

#[derive(Serialize, Deserialize, Default, Clone, Debug)]
struct AutomationsConfig {
    rules: Vec<Rule>,
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

fn main() {
    log_message("XOS Automation Engine starting...");
    
    // Track fired rules to avoid notification loops (cooldown of 30 seconds)
    let mut fired_rules: HashMap<String, Instant> = HashMap::new();

    let mut cached_config: Option<AutomationsConfig> = None;
    let mut cached_path: Option<String> = None;
    let mut last_modified: Option<std::time::SystemTime> = None;

    loop {
        let config_path = if Path::new(CONFIG_FILE).exists() {
            CONFIG_FILE
        } else {
            FALLBACK_CONFIG
        };

        let needs_reload = match &cached_path {
            Some(path) if path == config_path => {
                if let Ok(metadata) = fs::metadata(config_path) {
                    if let Ok(modified) = metadata.modified() {
                        Some(modified) != last_modified
                    } else {
                        true
                    }
                } else {
                    true
                }
            }
            _ => true,
        };

        let config = if needs_reload {
            let cfg = match read_config(config_path) {
                Ok(cfg) => cfg,
                Err(_) => {
                    let default_cfg = get_default_config();
                    let _ = write_config(config_path, &default_cfg);
                    default_cfg
                }
            };
            let mod_time = fs::metadata(config_path).and_then(|m| m.modified()).ok();
            cached_config = Some(cfg.clone());
            cached_path = Some(config_path.to_string());
            last_modified = mod_time;
            cfg
        } else {
            cached_config.as_ref().unwrap().clone()
        };

        // Gather current system states
        let metrics = gather_system_metrics();

        for rule in config.rules {
            if evaluate_trigger(&rule.trigger, &metrics) {
                let now = Instant::now();
                let should_fire = match fired_rules.get(&rule.name) {
                    Some(last_fired) => now.duration_since(*last_fired) > Duration::from_secs(30),
                    None => true,
                };

                if should_fire {
                    log_message(&format!("TRIGGER MATCHED: Rule '{}' (trigger: '{}')", rule.name, rule.trigger));
                    run_action(&rule.action);
                    fired_rules.insert(rule.name.clone(), now);
                }
            }
        }

        thread::sleep(Duration::from_secs(4));
    }
}

fn gather_system_metrics() -> HashMap<String, String> {
    let mut metrics = HashMap::new();
    let use_run_dir = check_run_dir();

    // 1. Parse battery
    let b_path = if use_run_dir {
        BATTERY_STATE
    } else {
        FALLBACK_BATTERY
    };

    if let Ok(content) = fs::read_to_string(b_path) {
        for line in content.lines() {
            let parts: Vec<&str> = line.split('=').collect();
            if parts.len() == 2 {
                metrics.insert(parts[0].to_string(), parts[1].to_string());
            }
        }
    }

    // 2. Parse thermal
    let t_path = if use_run_dir {
        THERMAL_STATE
    } else {
        FALLBACK_THERMAL
    };

    if let Ok(content) = fs::read_to_string(t_path) {
        for line in content.lines() {
            let parts: Vec<&str> = line.split('=').collect();
            if parts.len() == 2 {
                metrics.insert(parts[0].to_string(), parts[1].to_string());
            }
        }
    }

    metrics
}

fn evaluate_trigger(trigger: &str, metrics: &HashMap<String, String>) -> bool {
    let (key, op, target) = if let Some(idx) = trigger.find("==") {
        (&trigger[..idx], "==", &trigger[idx + 2..])
    } else if let Some(idx) = trigger.find('<') {
        (&trigger[..idx], "<", &trigger[idx + 1..])
    } else if let Some(idx) = trigger.find('>') {
        (&trigger[..idx], ">", &trigger[idx + 1..])
    } else {
        return false;
    };

    let key = key.trim();
    let target = target.trim();

    let Some(current_val) = metrics.get(key) else {
        return false;
    };

    let current_val = current_val.trim();

    match op {
        "<" => {
            let cur: f64 = current_val.parse().unwrap_or(0.0);
            let tgt: f64 = target.parse().unwrap_or(0.0);
            cur < tgt
        }
        ">" => {
            let cur: f64 = current_val.parse().unwrap_or(0.0);
            let tgt: f64 = target.parse().unwrap_or(0.0);
            cur > tgt
        }
        "==" => {
            current_val == target
        }
        _ => false,
    }
}

fn parse_command_line(cmd: &str) -> Option<(String, Vec<String>)> {
    let mut args = Vec::new();
    let mut current = String::new();
    let mut in_single_quote = false;
    let mut in_double_quote = false;
    let mut chars = cmd.chars();

    while let Some(c) = chars.next() {
        match c {
            '\'' if !in_double_quote => {
                in_single_quote = !in_single_quote;
            }
            '"' if !in_single_quote => {
                in_double_quote = !in_double_quote;
            }
            ' ' | '\t' if !in_single_quote && !in_double_quote => {
                if !current.is_empty() {
                    args.push(current);
                    current = String::new();
                }
            }
            _ => {
                current.push(c);
            }
        }
    }
    if !current.is_empty() {
        args.push(current);
    }

    if args.is_empty() {
        None
    } else {
        let program = args.remove(0);
        Some((program, args))
    }
}

fn run_action(action: &str) {
    log_message(&format!("Executing automation action: {}", action));
    
    let Some((program, args)) = parse_command_line(action) else {
        log_message("ERROR: Empty action command");
        return;
    };

    let output = Command::new(&program)
        .args(&args)
        .output();

    match output {
        Ok(out) => {
            if !out.status.success() {
                let err = String::from_utf8_lossy(&out.stderr);
                log_message(&format!("WARNING: Action command returned error: {}", err));
            }
        }
        Err(e) => {
            log_message(&format!("ERROR: Failed to run action command {}: {}", program, e));
        }
    }
}

fn read_config(path: &str) -> Result<AutomationsConfig, Box<dyn std::error::Error>> {
    let content = fs::read_to_string(path)?;
    let config: AutomationsConfig = toml::from_str(&content)?;
    Ok(config)
}

fn write_config(path: &str, config: &AutomationsConfig) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(parent) = Path::new(path).parent() {
        fs::create_dir_all(parent)?;
    }
    let content = toml::to_string(config)?;
    fs::write(path, content)?;
    Ok(())
}

fn get_default_config() -> AutomationsConfig {
    let rules = vec![
        Rule {
            name: "low_battery".to_string(),
            trigger: "capacity < 20".to_string(),
            action: "xos-notify 'Battery Low' 'System is running low on power (< 20%)'".to_string(),
        },
        Rule {
            name: "high_temp".to_string(),
            trigger: "cpu_temp > 85".to_string(),
            action: "xos-notify 'High Temperature' 'CPU temperature is critically high (> 85°C)'".to_string(),
        },
        Rule {
            name: "power_plugged".to_string(),
            trigger: "status == Charging".to_string(),
            action: "xos-notify 'Power Connected' 'Performance mode switched to Balanced'".to_string(),
        },
    ];
    AutomationsConfig { rules }
}
