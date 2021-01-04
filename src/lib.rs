//! Rust bindings to [OpenSlide](https://openslide.org/).
//!
//! This work has no affiliations with the official OpenSlide project.

use std::error::Error;
use std::fmt;

pub use openslide::{Address, OpenSlide, Region, Size};

mod openslide;
mod utils;

type Result<T> = std::result::Result<T, OpenSlideError>;

#[derive(Clone, PartialEq)]
pub enum OpenSlideError {
    MissingFile(String),
    UnsupportedFile(String),
    KeyError(String),
    IndexError(String),
    InternalError(String),
}

impl OpenSlideError {
    fn error_message(&self) -> String {
        match self {
            Self::MissingFile(m) => format!("File {} does not exist", m),
            Self::UnsupportedFile(m) => format!("Unsupported format: {}", m),
            Self::KeyError(m) => format!("Key {} does not exist", m),
            Self::IndexError(m) => format!("Level {} out of range", m),
            Self::InternalError(m) => m.to_string(),
        }
    }
}

impl Error for OpenSlideError {}

impl fmt::Debug for OpenSlideError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.error_message())
    }
}

impl fmt::Display for OpenSlideError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.error_message())
    }
}
