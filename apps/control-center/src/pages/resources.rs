use std::collections::HashMap;
use std::fs;
use std::path::Path;
use gtk4::prelude::*;
use gtk4::{Box as GtkBox, Label, Orientation, Scale};
use libadwaita as adw;
use adw::prelude::*;
use serde::{Deserialize, Serialize};

const CONFIG_FILE: &str = "/etc/xos/resource-budgets.toml";
const FALLBACK_CONFIG: &str = "/tmp/xos_resource_budgets.toml";

#[derive(Serialize, Deserialize, Clone, Debug)]
struct AppLimit {
    cpu_limit: u32,
    mem_limit: u32,
}

#[derive(Serialize, Deserialize, Default, Clone, Debug)]
struct ResourceConfig {
    apps: HashMap<String, AppLimit>,
}

pub struct ResourcesPage {
    pub container: adw::PreferencesPage,
}

impl ResourcesPage {
    pub fn new() -> Self {
        let page = adw::PreferencesPage::new();
        page.set_title("Resource Limits");
        page.set_icon_name(Some("utilities-system-monitor-symbolic"));

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

        for (app_name, limits) in config.apps {
            let group = adw::PreferencesGroup::builder()
                .title(&app_name)
                .build();

            let vbox = GtkBox::new(Orientation::Vertical, 8);
            vbox.set_margin_top(8);
            vbox.set_margin_bottom(8);
            vbox.set_margin_start(12);
            vbox.set_margin_end(12);

            // CPU Slider
            let cpu_lbl = Label::builder()
                .label(&format!("CPU Limit: {}%", limits.cpu_limit))
                .xalign(0.0)
                .build();

            let cpu_scale = Scale::with_range(Orientation::Horizontal, 10.0, 100.0, 5.0);
            cpu_scale.set_value(limits.cpu_limit as f64);
            cpu_scale.set_hexpand(true);

            // RAM Slider
            let mem_lbl = Label::builder()
                .label(&format!("Memory Limit: {} MB", limits.mem_limit))
                .xalign(0.0)
                .build();

            let mem_scale = Scale::with_range(Orientation::Horizontal, 256.0, 4096.0, 256.0);
            mem_scale.set_value(limits.mem_limit as f64);
            mem_scale.set_hexpand(true);

            // Connect change events
            let app_name_clone = app_name.clone();
            cpu_scale.connect_value_changed({
                let cpu_lbl = cpu_lbl.clone();
                let app_name = app_name_clone.clone();
                move |scale| {
                    let val = scale.value() as u32;
                    cpu_lbl.set_label(&format!("CPU Limit: {}%", val));
                    update_limit(&app_name, Some(val), None);
                }
            });

            mem_scale.connect_value_changed({
                let mem_lbl = mem_lbl.clone();
                let app_name = app_name_clone;
                move |scale| {
                    let val = scale.value() as u32;
                    mem_lbl.set_label(&format!("Memory Limit: {} MB", val));
                    update_limit(&app_name, None, Some(val));
                }
            });

            vbox.append(&cpu_lbl);
            vbox.append(&cpu_scale);
            vbox.append(&mem_lbl);
            vbox.append(&mem_scale);
            
            group.add(&vbox);
            page.add(&group);
        }

        Self { container: page }
    }
}

fn update_limit(app: &str, cpu: Option<u32>, mem: Option<u32>) {
    let config_path = if Path::new(CONFIG_FILE).exists() {
        CONFIG_FILE
    } else {
        FALLBACK_CONFIG
    };

    let mut config = read_config(config_path).unwrap_or_else(|_| get_default_config());
    if let Some(limit) = config.apps.get_mut(app) {
        if let Some(c) = cpu {
            limit.cpu_limit = c;
        }
        if let Some(m) = mem {
            limit.mem_limit = m;
        }
    }
    let _ = write_config(config_path, &config);
}

fn read_config(path: &str) -> Result<ResourceConfig, Box<dyn std::error::Error>> {
    let content = fs::read_to_string(path)?;
    let config: ResourceConfig = toml::from_str(&content)?;
    Ok(config)
}

fn write_config(path: &str, config: &ResourceConfig) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(parent) = Path::new(path).parent() {
        let _ = fs::create_dir_all(parent);
    }
    let content = toml::to_string(config)?;
    fs::write(path, content)?;
    Ok(())
}

fn get_default_config() -> ResourceConfig {
    let mut apps = HashMap::new();
    apps.insert(
        "firefox".to_string(),
        AppLimit {
            cpu_limit: 60,
            mem_limit: 2048,
        },
    );
    apps.insert(
        "foot".to_string(),
        AppLimit {
            cpu_limit: 20,
            mem_limit: 512,
        },
    );
    apps.insert(
        "xos-file-manager".to_string(),
        AppLimit {
            cpu_limit: 30,
            mem_limit: 1024,
        },
    );
    ResourceConfig { apps }
}
