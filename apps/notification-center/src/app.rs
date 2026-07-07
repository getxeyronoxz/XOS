use gtk4::prelude::*;
use libadwaita as adw;

use crate::window::NotificationCenterWindow;

const APP_ID: &str = "org.xos.NotificationCenter";

pub struct NotificationCenterApp;

impl NotificationCenterApp {
    pub fn run() -> Result<(), Box<dyn std::error::Error>> {
        let app = adw::Application::builder()
            .application_id(APP_ID)
            .build();

        app.connect_activate(|app| {
            if let Some(win) = app.active_window() {
                win.present();
                return;
            }
            let window = NotificationCenterWindow::new(app);
            window.present();
        });

        app.run();
        Ok(())
    }
}
