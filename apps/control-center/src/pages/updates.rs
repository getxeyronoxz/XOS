use std::fs;
use std::path::Path;
use gtk4::prelude::*;
use gtk4::{Box as GtkBox, Label, Orientation, Button, Spinner};
use libadwaita as adw;
use adw::prelude::*;

const STATE_FILE: &str = "/run/xos/updates.state";
const FALLBACK_STATE: &str = "/tmp/xos_updates.state";
const TRIGGER_FILE: &str = "/run/xos/update.trigger";
const FALLBACK_TRIGGER: &str = "/tmp/xos_update.trigger";

pub struct UpdatesPage {
    pub container: adw::PreferencesPage,
    status_lbl: Label,
    count_lbl: Label,
    last_check_lbl: Label,
    install_btn: Button,
    spinner: Spinner,
}

impl UpdatesPage {
    pub fn new() -> Self {
        let page = adw::PreferencesPage::new();
        page.set_title("System Updates");
        page.set_icon_name(Some("system-software-update-symbolic"));

        let group = adw::PreferencesGroup::builder()
            .title("Update Status")
            .build();

        let vbox = GtkBox::new(Orientation::Vertical, 12);
        vbox.set_margin_top(8);
        vbox.set_margin_bottom(8);
        vbox.set_margin_start(12);
        vbox.set_margin_end(12);

        let status_lbl = Label::builder()
            .label("Status: Idle")
            .xalign(0.0)
            .build();
        status_lbl.add_css_class("heading");

        let count_lbl = Label::builder()
            .label("Available Updates: 0 packages")
            .xalign(0.0)
            .build();

        let last_check_lbl = Label::builder()
            .label("Last Checked: Never")
            .xalign(0.0)
            .build();

        let hbox_controls = GtkBox::new(Orientation::Horizontal, 12);
        let check_btn = Button::builder().label("Check for Updates").build();
        let install_btn = Button::builder().label("Install Updates").build();
        install_btn.add_css_class("suggested-action");

        let spinner = Spinner::new();
        spinner.set_visible(false);

        check_btn.connect_clicked({
            let count_lbl = count_lbl.clone();
            move |_| {
                // Mock checking updates
                count_lbl.set_label("Available Updates: 4 packages (checking finished)");
            }
        });

        install_btn.connect_clicked({
            let spinner = spinner.clone();
            move |btn| {
                btn.set_sensitive(false);
                spinner.set_visible(true);
                spinner.start();
                trigger_upgrade();
            }
        });

        hbox_controls.append(&check_btn);
        hbox_controls.append(&install_btn);
        hbox_controls.append(&spinner);

        vbox.append(&status_lbl);
        vbox.append(&count_lbl);
        vbox.append(&last_check_lbl);
        vbox.append(&hbox_controls);

        group.add(&vbox);
        page.add(&group);

        let mut page_self = Self {
            container: page,
            status_lbl,
            count_lbl,
            last_check_lbl,
            install_btn,
            spinner,
        };

        page_self.update();
        page_self
    }

    pub fn update(&mut self) {
        let path = if Path::new(STATE_FILE).exists() {
            STATE_FILE
        } else {
            FALLBACK_STATE
        };

        let mut status = "Idle".to_string();
        let mut count = 0;
        let mut last_check = "Never".to_string();

        if let Ok(contents) = fs::read_to_string(path) {
            for line in contents.lines() {
                let parts: Vec<&str> = line.split('=').collect();
                if parts.len() == 2 {
                    match parts[0] {
                        "status" => status = parts[1].to_string(),
                        "updates_available" => count = parts[1].parse().unwrap_or(0),
                        "last_check" => last_check = parts[1].to_string(),
                        _ => {}
                    }
                }
            }
        }

        self.status_lbl.set_label(&format!("Status: {}", status));
        self.count_lbl.set_label(&format!("Available Updates: {} packages", count));
        self.last_check_lbl.set_label(&format!("Last Checked: {}", last_check));

        // Manage UI states based on active status
        if status == "Idle" {
            self.spinner.stop();
            self.spinner.set_visible(false);
            self.install_btn.set_sensitive(count > 0);
        } else {
            self.spinner.set_visible(true);
            self.spinner.start();
            self.install_btn.set_sensitive(false);
        }
    }
}

fn trigger_upgrade() {
    let path = if Path::new("/run/xos").exists() {
        TRIGGER_FILE
    } else {
        FALLBACK_TRIGGER
    };

    let _ = fs::write(path, "upgrade\n");
}
