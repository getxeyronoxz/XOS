//! XOS Screenshot — Phase 2 v1
//! Region and full-screen capture via grim/slurp.

mod app;
mod capture;
mod error;
mod window;

use std::env;

use crate::app::ScreenshotApp;
use crate::capture::{CaptureMode, CaptureTarget, capture};
use crate::error::ScreenshotError;

fn main() -> Result<(), ScreenshotError> {
    let args: Vec<String> = env::args().collect();

    if args.len() > 1 {
        return run_cli(&args[1..]);
    }

    ScreenshotApp::run()
}

fn run_cli(args: &[String]) -> Result<(), ScreenshotError> {
    let mut mode = CaptureMode::Region;
    let mut target = CaptureTarget::Clipboard;

    for arg in args {
        match arg.as_str() {
            "--region" => mode = CaptureMode::Region,
            "--full" | "--fullscreen" => mode = CaptureMode::FullScreen,
            "--clipboard" | "-c" => target = CaptureTarget::Clipboard,
            "--save" | "-s" => target = CaptureTarget::Save,
            "--help" | "-h" => {
                print_help();
                return Ok(());
            }
            other => {
                return Err(ScreenshotError::Capture(format!(
                    "Unknown argument: {other}"
                )));
            }
        }
    }

    capture(mode, target).map_err(ScreenshotError::Capture)
}

fn print_help() {
    println!(
        "xos-screenshot — XOS screenshot tool\n\n\
         Usage:\n  \
           xos-screenshot [--region|--full] [--clipboard|--save]\n\n\
         Options:\n  \
           --region       Capture selected region (default)\n  \
           --full         Capture full screen\n  \
           --clipboard    Copy PNG to clipboard (default)\n  \
           --save         Save PNG to ~/Pictures/\n\n\
         Run without arguments to open the GUI."
    );
}
