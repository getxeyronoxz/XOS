use std::fmt;

#[derive(Debug)]
pub enum ControlCenterError {
    AppRun(String),
}

impl fmt::Display for ControlCenterError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::AppRun(msg) => write!(f, "Control Center failed: {msg}"),
        }
    }
}

impl std::error::Error for ControlCenterError {}
