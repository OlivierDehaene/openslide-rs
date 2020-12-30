use std::error;
use std::fmt;

pub use deepzoom::DeepZoom;
pub use openslide::{Address, OpenSlide, Region, Size};
use std::fmt::Formatter;

mod deepzoom;
mod openslide;
mod utils;

type Result<T> = std::result::Result<T, OpenSlideError>;

#[derive(Clone, PartialEq)]
pub enum OpenSlideError {
    MissingFile(String),
    UnsupportedFile(String),
    KeyError(String),
    InternalError(String),
}

impl OpenSlideError {
    fn error_message(&self) -> String {
        match self {
            Self::MissingFile(m) => format!("MissingFile: {}", m),
            Self::UnsupportedFile(m) => format!("UnsupportedFile: {}", m),
            Self::KeyError(m) => format!("KeyError: {}", m),
            Self::InternalError(m) => format!("InternalError: {}", m),
        }
    }
}

impl fmt::Debug for OpenSlideError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.error_message())
    }
}

impl fmt::Display for OpenSlideError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.error_message())
    }
}
