use gtk4::prelude::*;
use libadwaita as adw;

use crate::error::FileManagerError;
use crate::window::FileManagerWindow;

const APP_ID: &str = "org.xos.FileManager";

pub struct FileManagerApp;

impl FileManagerApp {
    pub fn run() -> Result<(), FileManagerError> {
        let app = adw::Application::builder()
            .application_id(APP_ID)
            .build();

        app.connect_activate(|app| {
            if let Some(win) = app.active_window() {
                win.present();
                return;
            }
            let window = FileManagerWindow::new(app);
            window.present();
        });

        app.run();
        Ok(())
    }
}
