//! XOS Control Center — Phase 2 v1
//! Appearance, display, sound, and network settings.

mod app;
mod error;
mod pages;
mod window;

use crate::app::ControlCenterApp;
use crate::error::ControlCenterError;

fn main() -> Result<(), ControlCenterError> {
    ControlCenterApp::run()
}
