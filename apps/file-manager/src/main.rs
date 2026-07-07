//! XOS File Manager — Phase 2
//! Browse, open, copy, move, delete, preview, and search.

mod app;
mod breadcrumb;
mod error;
mod listing;
mod operations;
mod preview;
mod search;
mod search_bar;
mod sidebar;
mod window;

use crate::app::FileManagerApp;
use crate::error::FileManagerError;

fn main() -> Result<(), FileManagerError> {
    FileManagerApp::run()
}
