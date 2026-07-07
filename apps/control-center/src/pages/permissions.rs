use std::collections::HashMap;
use std::fs;
use std::path::Path;
use gtk4::prelude::*;
use gtk4::{Box as GtkBox, Orientation, Switch};
use libadwaita as adw;
use adw::prelude::*;
use serde::{Deserialize, Serialize};

const CONFIG_FILE: &str = "/etc/xos/permissions.toml";
const FALLBACK_CONFIG: &str = "/tmp/xos_permissions.toml";

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

pub struct PermissionsPage {
    pub container: adw::PreferencesPage,
}

impl PermissionsPage {
    pub fn new() -> Self {
        let page = adw::PreferencesPage::new();
        page.set_title("App Permissions");
        page.set_icon_name(Some("security-high-symbolic"));

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

        for (app_name, perms) in config.apps {
            let group = adw::PreferencesGroup::builder()
                .title(&app_name)
                .build();

            let vbox = GtkBox::new(Orientation::Vertical, 6);
            vbox.set_margin_top(8);
            vbox.set_margin_bottom(8);
            vbox.set_margin_start(12);
            vbox.set_margin_end(12);

            // 1. Camera Toggle
            let cam_row = adw::ActionRow::builder()
                .title("Camera Access")
                .subtitle("Allow app to access hardware webcam")
                .build();
            let cam_switch = Switch::builder().active(perms.camera).build();
            cam_row.add_suffix(&cam_switch);
            cam_row.set_activatable_widget(Some(&cam_switch));

            // 2. Microphone Toggle
            let mic_row = adw::ActionRow::builder()
                .title("Microphone Access")
                .subtitle("Allow app to record audio")
                .build();
            let mic_switch = Switch::builder().active(perms.microphone).build();
            mic_row.add_suffix(&mic_switch);
            mic_row.set_activatable_widget(Some(&mic_switch));

            // 3. Location Toggle
            let loc_row = adw::ActionRow::builder()
                .title("Location Services")
                .subtitle("Allow app to read geographic location")
                .build();
            let loc_switch = Switch::builder().active(perms.location).build();
            loc_row.add_suffix(&loc_switch);
            loc_row.set_activatable_widget(Some(&loc_switch));

            // 4. Network Toggle
            let net_row = adw::ActionRow::builder()
                .title("Network Access")
                .subtitle("Allow app to make outbound connections")
                .build();
            let net_switch = Switch::builder().active(perms.network).build();
            net_row.add_suffix(&net_switch);
            net_row.set_activatable_widget(Some(&net_switch));

            // Connect change events
            let app_name_clone = app_name.clone();
            cam_switch.connect_state_set({
                let app = app_name_clone.clone();
                move |_, state| {
                    update_permission(&app, "camera", state);
                    glib::Propagation::Proceed
                }
            });

            mic_switch.connect_state_set({
                let app = app_name_clone.clone();
                move |_, state| {
                    update_permission(&app, "microphone", state);
                    glib::Propagation::Proceed
                }
            });

            loc_switch.connect_state_set({
                let app = app_name_clone.clone();
                move |_, state| {
                    update_permission(&app, "location", state);
                    glib::Propagation::Proceed
                }
            });

            net_switch.connect_state_set({
                let app = app_name_clone;
                move |_, state| {
                    update_permission(&app, "network", state);
                    glib::Propagation::Proceed
                }
            });

            vbox.append(&cam_row);
            vbox.append(&mic_row);
            vbox.append(&loc_row);
            vbox.append(&net_row);

            group.add(&vbox);
            page.add(&group);
        }

        Self { container: page }
    }
}

fn update_permission(app: &str, field: &str, value: bool) {
    let config_path = if Path::new(CONFIG_FILE).exists() {
        CONFIG_FILE
    } else {
        FALLBACK_CONFIG
    };

    let mut config = read_config(config_path).unwrap_or_else(|_| get_default_config());
    if let Some(perms) = config.apps.get_mut(app) {
        match field {
            "camera" => perms.camera = value,
            "microphone" => perms.microphone = value,
            "location" => perms.location = value,
            "network" => perms.network = value,
            _ => {}
        }
    }
    let _ = write_config(config_path, &config);
}

fn read_config(path: &str) -> Result<PermissionsConfig, Box<dyn std::error::Error>> {
    let content = fs::read_to_string(path)?;
    let config: PermissionsConfig = toml::from_str(&content)?;
    Ok(config)
}

fn write_config(path: &str, config: &PermissionsConfig) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(parent) = Path::new(path).parent() {
        let _ = fs::create_dir_all(parent);
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
