mod app;
mod overview;
mod processes;
mod sysdata;
mod window;

use app::SystemMonitorApp;

fn main() {
    if let Err(err) = SystemMonitorApp::run() {
        eprintln!("System Monitor application error: {err}");
        std::process::exit(1);
    }
}
