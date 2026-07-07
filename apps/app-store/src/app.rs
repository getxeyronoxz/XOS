use gtk4::prelude::*;
use libadwaita as adw;
use std::process::ExitCode;

use crate::window::AppStoreWindow;

pub struct AppStoreApp {
    app: adw::Application,
}

impl AppStoreApp {
    pub fn new() -> Self {
        let app = adw::Application::builder()
            .application_id("org.xos.AppStore")
            .build();

        app.connect_activate(|app| {
            let win = AppStoreWindow::new(app);
            win.present();
        });

        Self { app }
    }

    pub fn run(&self) -> ExitCode {
        let args: Vec<String> = std::env::args().collect();
        let status = self.app.run_with_args(&args);
        ExitCode::from(status.value() as u8)
    }
}
