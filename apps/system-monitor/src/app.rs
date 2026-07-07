use gtk4::prelude::*;
use libadwaita as adw;

use crate::window::SystemMonitorWindow;

const APP_ID: &str = "org.xos.SystemMonitor";

pub struct SystemMonitorApp;

impl SystemMonitorApp {
    pub fn run() -> Result<(), Box<dyn std::error::Error>> {
        let app = adw::Application::builder()
            .application_id(APP_ID)
            .build();

        app.connect_activate(|app| {
            if let Some(win) = app.active_window() {
                win.present();
                return;
            }
            let window = SystemMonitorWindow::new(app);
            window.present();
        });

        app.run();
        Ok(())
    }
}
