mod app;
mod window;

use std::process::ExitCode;

fn main() -> ExitCode {
    let app = app::AutomationBuilderApp::new();
    app.run()
}
