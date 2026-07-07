use gtk4::prelude::*;
use libadwaita as adw;

use crate::error::ControlCenterError;
use crate::window::ControlCenterWindow;

const APP_ID: &str = "org.xos.ControlCenter";

pub struct ControlCenterApp;

impl ControlCenterApp {
    pub fn run() -> Result<(), ControlCenterError> {
        let app = adw::Application::builder()
            .application_id(APP_ID)
            .build();

        app.connect_activate(|app| {
            if let Some(win) = app.active_window() {
                win.present();
                return;
            }
            let window = ControlCenterWindow::new(app);
            window.present();
        });

        app.run();
        Ok(())
    }
}
