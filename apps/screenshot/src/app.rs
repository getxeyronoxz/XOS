use gtk4::prelude::*;
use libadwaita as adw;

use crate::error::ScreenshotError;
use crate::window::ScreenshotWindow;

const APP_ID: &str = "org.xos.Screenshot";

pub struct ScreenshotApp;

impl ScreenshotApp {
    pub fn run() -> Result<(), ScreenshotError> {
        let app = adw::Application::builder()
            .application_id(APP_ID)
            .build();

        app.connect_activate(|app| {
            if let Some(win) = app.active_window() {
                win.present();
                return;
            }
            let window = ScreenshotWindow::new(app);
            window.present();
        });

        app.run();
        Ok(())
    }
}
