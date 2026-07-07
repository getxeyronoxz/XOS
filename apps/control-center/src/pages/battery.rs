use std::fs;
use std::path::Path;
use gtk4::prelude::*;
use gtk4::{Box as GtkBox, Label, Orientation, ProgressBar, Button};
use libadwaita as adw;
use adw::prelude::*;

const STATE_FILE: &str = "/run/xos/battery.state";
const FALLBACK_STATE_FILE: &str = "/tmp/xos_battery.state";
const CONFIG_FILE: &str = "/etc/xos/battery.conf";
const FALLBACK_CONFIG_FILE: &str = "/tmp/xos_battery.conf";

pub struct BatteryPage {
    pub container: adw::PreferencesPage,
    status_label: Label,
    capacity_bar: ProgressBar,
    health_label: Label,
    cycle_label: Label,
    rate_label: Label,
    btn_80: Button,
    btn_90: Button,
    btn_100: Button,
}

impl BatteryPage {
    pub fn new() -> Self {
        let page = adw::PreferencesPage::new();
        page.set_title("Battery");
        page.set_icon_name(Some("battery-level-100-charged-symbolic"));

        // 1. Status Section
        let status_group = adw::PreferencesGroup::builder()
            .title("Battery Status")
            .build();

        let status_box = GtkBox::new(Orientation::Vertical, 12);
        status_box.set_margin_top(8);
        status_box.set_margin_bottom(8);
        status_box.set_margin_start(12);
        status_box.set_margin_end(12);

        let status_label = Label::builder()
            .label("Battery: Unknown")
            .xalign(0.0)
            .build();
        status_label.add_css_class("heading");

        let capacity_bar = ProgressBar::new();
        capacity_bar.set_show_text(true);

        let rate_label = Label::builder()
            .label("Charge Rate: 0.0 W")
            .xalign(0.0)
            .build();

        status_box.append(&status_label);
        status_box.append(&capacity_bar);
        status_box.append(&rate_label);
        status_group.add(&status_box);
        page.add(&status_group);

        // 2. Health Section
        let health_group = adw::PreferencesGroup::builder()
            .title("Battery Health & Details")
            .build();

        let health_box = GtkBox::new(Orientation::Vertical, 8);
        health_box.set_margin_top(8);
        health_box.set_margin_bottom(8);
        health_box.set_margin_start(12);
        health_box.set_margin_end(12);

        let health_label = Label::builder()
            .label("Battery Health: 100.0%")
            .xalign(0.0)
            .build();
        health_label.add_css_class("heading");

        let cycle_label = Label::builder()
            .label("Cycle Count: 0")
            .xalign(0.0)
            .build();

        health_box.append(&health_label);
        health_box.append(&cycle_label);
        health_group.add(&health_box);
        page.add(&health_group);

        // 3. Charge Limit Section
        let limit_group = adw::PreferencesGroup::builder()
            .title("Charge Limit Setting")
            .description("Limiting full charge capacity extends overall battery lifespan.")
            .build();

        let limit_hbox = GtkBox::new(Orientation::Horizontal, 12);
        limit_hbox.set_margin_top(8);
        limit_hbox.set_margin_bottom(8);
        limit_hbox.set_margin_start(12);
        limit_hbox.set_margin_end(12);

        let limit_label = Label::builder()
            .label("Maximum Charge Limit:")
            .xalign(0.0)
            .hexpand(true)
            .build();

        let btn_80 = Button::builder().label("80%").build();
        let btn_90 = Button::builder().label("90%").build();
        let btn_100 = Button::builder().label("100%").build();

        btn_80.connect_clicked(|_| set_charge_limit(80));
        btn_90.connect_clicked(|_| set_charge_limit(90));
        btn_100.connect_clicked(|_| set_charge_limit(100));

        limit_hbox.append(&limit_label);
        limit_hbox.append(&btn_80);
        limit_hbox.append(&btn_90);
        limit_hbox.append(&btn_100);
        limit_group.add(&limit_hbox);
        page.add(&limit_group);

        let mut page_self = Self {
            container: page,
            status_label,
            capacity_bar,
            health_label,
            cycle_label,
            rate_label,
            btn_80,
            btn_90,
            btn_100,
        };

        page_self.update();
        page_self
    }

    pub fn update(&mut self) {
        // Find state file
        let path = if Path::new(STATE_FILE).exists() {
            STATE_FILE
        } else {
            FALLBACK_STATE_FILE
        };

        let mut capacity = 0;
        let mut status = "Unknown".to_string();
        let mut watts = 0.0;
        let mut cycles = 0;
        let mut health = 100.0;
        let mut charge_limit = 100;

        if let Ok(contents) = fs::read_to_string(path) {
            for line in contents.lines() {
                let parts: Vec<&str> = line.split('=').collect();
                if parts.len() == 2 {
                    match parts[0] {
                        "capacity" => capacity = parts[1].parse().unwrap_or(0),
                        "status" => status = parts[1].to_string(),
                        "watts" => watts = parts[1].parse().unwrap_or(0.0),
                        "cycles" => cycles = parts[1].parse().unwrap_or(0),
                        "health" => health = parts[1].parse().unwrap_or(100.0),
                        "charge_limit" => charge_limit = parts[1].parse().unwrap_or(100),
                        _ => {}
                    }
                }
            }
        }

        self.status_label.set_label(&format!("Battery: {} ({}%)", status, capacity));
        self.capacity_bar.set_fraction(capacity as f64 / 100.0);
        self.rate_label.set_label(&format!("Charge Rate: {:.1} W", watts));
        self.health_label.set_label(&format!("Battery Health: {:.1}%", health));
        self.cycle_label.set_label(&format!("Cycle Count: {}", cycles));

        // Highlight active charge limit button
        self.btn_80.remove_css_class("suggested-action");
        self.btn_90.remove_css_class("suggested-action");
        self.btn_100.remove_css_class("suggested-action");

        match charge_limit {
            80 => self.btn_80.add_css_class("suggested-action"),
            90 => self.btn_90.add_css_class("suggested-action"),
            _ => self.btn_100.add_css_class("suggested-action"),
        }
    }
}

fn set_charge_limit(limit: i32) {
    // Write limit config
    let content = format!("charge_limit={}\n", limit);
    if fs::create_dir_all("/etc/xos").is_ok() {
        if fs::write(CONFIG_FILE, &content).is_ok() {
            return;
        }
    }
    let _ = fs::write(FALLBACK_CONFIG_FILE, &content);
}
