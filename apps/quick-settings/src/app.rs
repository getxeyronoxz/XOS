use gtk4::prelude::*;
use libadwaita as adw;

use crate::window::QuickSettingsWindow;

const APP_ID: &str = "org.xos.QuickSettings";

pub struct QuickSettingsApp;

impl QuickSettingsApp {
    pub fn run() {
        let app = adw::Application::builder()
            .application_id(APP_ID)
            .build();

        app.connect_activate(|app| {
            if let Some(win) = app.active_window() {
                win.present();
                return;
            }
            let window = QuickSettingsWindow::new(app);
            window.present();
        });

        app.run();
    }
}
