use std::fmt;

#[derive(Debug)]
pub enum ArchiveError {
    Storage(String),
    AppRun(String),
    Operation(String),
}

impl std::error::Error for ArchiveError {}

impl fmt::Display for ArchiveError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ArchiveError::Storage(s) => write!(f, "Storage error: {s}"),
            ArchiveError::AppRun(s) => write!(f, "App run error: {s}"),
            ArchiveError::Operation(s) => write!(f, "Operation error: {s}"),
        }
    }
}
