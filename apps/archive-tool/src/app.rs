use gtk4::prelude::*;
use libadwaita as adw;

use crate::error::ArchiveError;
use crate::window::ArchiveWindow;

const APP_ID: &str = "org.xos.ArchiveTool";

pub struct ArchiveApp;

impl ArchiveApp {
    pub fn run() -> Result<(), ArchiveError> {
        let app = adw::Application::builder()
            .application_id(APP_ID)
            .build();

        app.connect_activate(|app| {
            if let Some(win) = app.active_window() {
                win.present();
                return;
            }
            let window = ArchiveWindow::new(app);
            window.present();
        });

        app.run();
        Ok(())
    }
}
