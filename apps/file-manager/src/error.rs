use std::fmt;

#[derive(Debug)]
pub enum FileManagerError {
    GtkInit(String),
    AppRun(String),
    Operation(String),
}

impl fmt::Display for FileManagerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::GtkInit(msg) => write!(f, "GTK init failed: {msg}"),
            Self::AppRun(msg) => write!(f, "Application run failed: {msg}"),
            Self::Operation(msg) => write!(f, "File operation failed: {msg}"),
        }
    }
}

impl std::error::Error for FileManagerError {}
