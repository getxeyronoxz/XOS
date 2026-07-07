//! XOS Notes — Phase 2 v1
//! Simple note list and text editor.

mod app;
mod error;
mod storage;
mod window;

use crate::app::NotesApp;
use crate::error::NotesError;

fn main() -> Result<(), NotesError> {
    NotesApp::run()
}
