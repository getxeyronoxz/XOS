use gtk4::prelude::*;
use libadwaita as adw;

use crate::error::NotesError;
use crate::window::NotesWindow;

const APP_ID: &str = "org.xos.Notes";

pub struct NotesApp;

impl NotesApp {
    pub fn run() -> Result<(), NotesError> {
        let app = adw::Application::builder()
            .application_id(APP_ID)
            .build();

        app.connect_activate(|app| {
            if let Some(win) = app.active_window() {
                win.present();
                return;
            }
            let window = NotesWindow::new(app);
            window.present();
        });

        app.run();
        Ok(())
    }
}
