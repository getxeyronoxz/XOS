use std::fs;
use std::path::Path;
use gtk4::prelude::*;
use gtk4::{Box as GtkBox, Label, Orientation, ProgressBar, Button, Scale, Switch};
use libadwaita as adw;
use adw::prelude::*;

const THERMAL_STATE_FILE: &str = "/run/xos/thermal.state";
const FALLBACK_THERMAL_STATE: &str = "/tmp/xos_thermal.state";
const FAN_STATE_FILE: &str = "/run/xos/fan.state";
const FALLBACK_FAN_STATE: &str = "/tmp/xos_fan.state";
const FAN_CONFIG_FILE: &str = "/etc/xos/fan.conf";
const FALLBACK_FAN_CONFIG: &str = "/tmp/xos_fan.conf";

pub struct HeatPage {
    pub container: adw::PreferencesPage,
    cpu_lbl: Label,
    cpu_bar: ProgressBar,
    gpu_lbl: Label,
    gpu_bar: ProgressBar,
    nvme_lbl: Label,
    nvme_bar: ProgressBar,
    fan_lbl: Label,
    fan_bar: ProgressBar,
    throttle_lbl: Label,
}

impl HeatPage {
    pub fn new() -> Self {
        let page = adw::PreferencesPage::new();
        page.set_title("Heat & Cooling");
        page.set_icon_name(Some("weather-clear-symbolic"));

        // 1. Temperatures Group
        let temps_group = adw::PreferencesGroup::builder()
            .title("Temperatures")
            .build();

        let temps_vbox = GtkBox::new(Orientation::Vertical, 12);
        temps_vbox.set_margin_top(8);
        temps_vbox.set_margin_bottom(8);
        temps_vbox.set_margin_start(12);
        temps_vbox.set_margin_end(12);

        let cpu_lbl = Label::builder().label("CPU Temperature: 0.0°C").xalign(0.0).build();
        let cpu_bar = ProgressBar::new();
        
        let gpu_lbl = Label::builder().label("GPU Temperature: 0.0°C").xalign(0.0).build();
        let gpu_bar = ProgressBar::new();

        let nvme_lbl = Label::builder().label("NVMe SSD Temperature: 0.0°C").xalign(0.0).build();
        let nvme_bar = ProgressBar::new();

        let throttle_lbl = Label::builder().label("Thermal Throttling: INACTIVE").xalign(0.0).build();
        throttle_lbl.add_css_class("heading");

        temps_vbox.append(&cpu_lbl);
        temps_vbox.append(&cpu_bar);
        temps_vbox.append(&gpu_lbl);
        temps_vbox.append(&gpu_bar);
        temps_vbox.append(&nvme_lbl);
        temps_vbox.append(&nvme_bar);
        temps_vbox.append(&throttle_lbl);
        temps_group.add(&temps_vbox);
        page.add(&temps_group);

        // 2. Fan Group
        let fan_group = adw::PreferencesGroup::builder()
            .title("Fan Control")
            .build();

        let fan_vbox = GtkBox::new(Orientation::Vertical, 12);
        fan_vbox.set_margin_top(8);
        fan_vbox.set_margin_bottom(8);
        fan_vbox.set_margin_start(12);
        fan_vbox.set_margin_end(12);

        let fan_lbl = Label::builder().label("Active Fan Speed: 0% (0 RPM)").xalign(0.0).build();
        let fan_bar = ProgressBar::new();

        let manual_row = adw::ActionRow::builder()
            .title("Manual Fan Speed Control")
            .subtitle("Override automatic fan curve")
            .build();

        let manual_switch = Switch::new();
        manual_row.add_suffix(&manual_switch);
        manual_row.set_activatable_widget(Some(&manual_switch));

        let speed_scale = Scale::with_range(Orientation::Horizontal, 0.0, 100.0, 1.0);
        speed_scale.set_value(50.0);
        speed_scale.set_hexpand(true);

        manual_switch.connect_state_set({
            let scale = speed_scale.clone();
            move |_, state| {
                scale.set_sensitive(state);
                if state {
                    set_fan_override(scale.value() as i32);
                } else {
                    set_fan_override(-1); // Auto
                }
                glib::Propagation::Proceed
            }
        });

        speed_scale.connect_value_changed({
            let sw = manual_switch.clone();
            move |scale| {
                if sw.is_active() {
                    set_fan_override(scale.value() as i32);
                }
            }
        });

        // Initialize state
        speed_scale.set_sensitive(false);

        fan_vbox.append(&fan_lbl);
        fan_vbox.append(&fan_bar);
        fan_vbox.append(&manual_row);
        fan_vbox.append(&speed_scale);
        fan_group.add(&fan_vbox);
        page.add(&fan_group);

        // 3. Thermal Protection Group
        let prot_group = adw::PreferencesGroup::builder()
            .title("Thermal Protection Settings")
            .description("Automatically throttle or shutdown if temperatures exceed critical levels.")
            .build();

        let prot_row = adw::ActionRow::builder()
            .title("Critical Temperature Limit")
            .subtitle("85°C (Recommended)")
            .build();
        
        let select_btn = Button::builder().label("Change").build();
        prot_row.add_suffix(&select_btn);
        prot_group.add(&prot_row);
        page.add(&prot_group);

        let mut page_self = Self {
            container: page,
            cpu_lbl,
            cpu_bar,
            gpu_lbl,
            gpu_bar,
            nvme_lbl,
            nvme_bar,
            fan_lbl,
            fan_bar,
            throttle_lbl,
        };

        page_self.update();
        page_self
    }

    pub fn update(&mut self) {
        // Temps
        let t_path = if Path::new(THERMAL_STATE_FILE).exists() {
            THERMAL_STATE_FILE
        } else {
            FALLBACK_THERMAL_STATE
        };

        let mut cpu: f64 = 40.0;
        let mut gpu: f64 = 38.0;
        let mut nvme: f64 = 34.0;
        let mut throttle = false;

        if let Ok(contents) = fs::read_to_string(t_path) {
            for line in contents.lines() {
                let parts: Vec<&str> = line.split('=').collect();
                if parts.len() == 2 {
                    match parts[0] {
                        "cpu_temp" => cpu = parts[1].parse().unwrap_or(40.0),
                        "gpu_temp" => gpu = parts[1].parse().unwrap_or(38.0),
                        "nvme_temp" => nvme = parts[1].parse().unwrap_or(34.0),
                        "throttle" => throttle = parts[1] == "1",
                        _ => {}
                    }
                }
            }
        }

        self.cpu_lbl.set_label(&format!("CPU Temperature: {:.1}°C", cpu));
        self.cpu_bar.set_fraction((cpu / 100.0).clamp(0.0, 1.0));
        
        self.gpu_lbl.set_label(&format!("GPU Temperature: {:.1}°C", gpu));
        self.gpu_bar.set_fraction((gpu / 100.0).clamp(0.0, 1.0));

        self.nvme_lbl.set_label(&format!("NVMe SSD Temperature: {:.1}°C", nvme));
        self.nvme_bar.set_fraction((nvme / 100.0).clamp(0.0, 1.0));

        if throttle {
            self.throttle_lbl.set_label("Thermal Throttling: ACTIVE");
            self.throttle_lbl.add_css_class("error");
        } else {
            self.throttle_lbl.set_label("Thermal Throttling: INACTIVE");
            self.throttle_lbl.remove_css_class("error");
        }

        // Fan
        let f_path = if Path::new(FAN_STATE_FILE).exists() {
            FAN_STATE_FILE
        } else {
            FALLBACK_FAN_STATE
        };

        let mut speed = 0;
        let mut rpm = 0;

        if let Ok(contents) = fs::read_to_string(f_path) {
            for line in contents.lines() {
                let parts: Vec<&str> = line.split('=').collect();
                if parts.len() == 2 {
                    match parts[0] {
                        "fan_speed" => speed = parts[1].parse().unwrap_or(0),
                        "fan_rpm" => rpm = parts[1].parse().unwrap_or(0),
                        _ => {}
                    }
                }
            }
        }

        self.fan_lbl.set_label(&format!("Active Fan Speed: {}% ({} RPM)", speed, rpm));
        self.fan_bar.set_fraction(speed as f64 / 100.0);
    }
}

fn set_fan_override(speed: i32) {
    let content = format!("manual_speed={}\n", speed);
    if fs::create_dir_all("/etc/xos").is_ok() {
        if fs::write(FAN_CONFIG_FILE, &content).is_ok() {
            return;
        }
    }
    let _ = fs::write(FALLBACK_FAN_CONFIG, &content);
}
