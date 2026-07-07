//! XOS Quick Settings — compact panel for common system toggles.

mod app;
mod power;
mod sliders;
mod toggles;
mod window;

use crate::app::QuickSettingsApp;

fn main() {
    QuickSettingsApp::run();
}
