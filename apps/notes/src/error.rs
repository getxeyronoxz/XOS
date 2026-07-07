use std::fmt;

#[derive(Debug)]
pub enum NotesError {
    Storage(String),
    AppRun(String),
}

impl fmt::Display for NotesError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Storage(msg) => write!(f, "Notes storage error: {msg}"),
            Self::AppRun(msg) => write!(f, "Notes app error: {msg}"),
        }
    }
}

impl std::error::Error for NotesError {}
