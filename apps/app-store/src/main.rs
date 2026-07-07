mod app;
mod window;

use std::process::ExitCode;

fn main() -> ExitCode {
    let app = app::AppStoreApp::new();
    app.run()
}
