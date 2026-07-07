mod app;
mod archive;
mod error;
mod window;

use app::ArchiveApp;

fn main() {
    if let Err(err) = ArchiveApp::run() {
        eprintln!("Application error: {err}");
        std::process::exit(1);
    }
}
