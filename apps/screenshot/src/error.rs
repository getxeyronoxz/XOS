use std::fmt;

#[derive(Debug)]
pub enum ScreenshotError {
    Capture(String),
}

impl fmt::Display for ScreenshotError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Capture(msg) => write!(f, "{msg}"),
        }
    }
}

impl std::error::Error for ScreenshotError {}
