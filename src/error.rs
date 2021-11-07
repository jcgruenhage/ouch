//! Error types definitions.
//!
//! All usage errors will pass throught the Error enum, a lot of them in the Error::Custom.

use std::fmt::{self, Display};

use crate::utils::colors::*;

#[allow(missing_docs)]
/// All errors that can be generated by `ouch`
#[derive(Debug, PartialEq)]
pub enum Error {
    /// Not every IoError, some of them get filtered by `From<io::Error>` into other variants
    IoError { reason: String },
    /// Detected from io::Error if .kind() is io::ErrorKind::NotFound
    NotFound { error_title: String },
    /// NEEDS MORE CONTEXT
    AlreadyExists { error_title: String },
    /// From zip::result::ZipError::InvalidArchive
    InvalidZipArchive(&'static str),
    /// Detected from io::Error if .kind() is io::ErrorKind::PermissionDenied
    PermissionDenied { error_title: String },
    /// From zip::result::ZipError::UnsupportedArchive
    UnsupportedZipArchive(&'static str),
    /// TO BE REMOVED
    CompressingRootFolder,
    /// Specialized walkdir's io::Error wrapper with additional information on the error
    DirWalkError { reason: String },
    /// Custom and unique errors are reported in this variant
    Custom { reason: FinalError },
}

/// Alias to std's Result with ouch's Error
pub type Result<T> = std::result::Result<T, Error>;

/// Pretty final error message for end users, crashing the program after display.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct FinalError {
    /// Should be made of just one line, appears after the "[ERROR]" part
    title: String,
    /// Shown as a unnumbered list in yellow
    details: Vec<String>,
    /// Shown as green at the end to give hints on how to work around this error, if it's fixable
    hints: Vec<String>,
}

impl Display for FinalError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Title
        writeln!(f, "{}[ERROR]{} {}", *RED, *RESET, self.title)?;

        // Details
        for detail in &self.details {
            writeln!(f, " {}-{} {}", *WHITE, *YELLOW, detail)?;
        }

        // Hints
        if !self.hints.is_empty() {
            // Separate by one blank line.
            writeln!(f)?;
            for hint in &self.hints {
                writeln!(f, "{}hint:{} {}", *GREEN, *RESET, hint)?;
            }
        }

        write!(f, "{}", *RESET)
    }
}

impl FinalError {
    /// Only constructor
    pub fn with_title(title: impl ToString) -> Self {
        Self { title: title.to_string(), details: vec![], hints: vec![] }
    }

    /// Add one detail line, can have multiple
    pub fn detail(mut self, detail: impl ToString) -> Self {
        self.details.push(detail.to_string());
        self
    }

    /// Add one hint line, can have multiple
    pub fn hint(mut self, hint: impl ToString) -> Self {
        self.hints.push(hint.to_string());
        self
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let err = match self {
            Error::DirWalkError { reason } => FinalError::with_title(reason),
            Error::NotFound { error_title } => FinalError::with_title(error_title).detail("File not found"),
            Error::CompressingRootFolder => {
                FinalError::with_title("It seems you're trying to compress the root folder.")
                    .detail("This is unadvisable since ouch does compressions in-memory.")
                    .hint("Use a more appropriate tool for this, such as rsync.")
            }
            Error::IoError { reason } => FinalError::with_title(reason),
            Error::AlreadyExists { error_title } => FinalError::with_title(error_title).detail("File already exists"),
            Error::InvalidZipArchive(reason) => FinalError::with_title("Invalid zip archive").detail(reason),
            Error::PermissionDenied { error_title } => FinalError::with_title(error_title).detail("Permission denied"),
            Error::UnsupportedZipArchive(reason) => FinalError::with_title("Unsupported zip archive").detail(reason),
            Error::Custom { reason } => reason.clone(),
        };

        write!(f, "{}", err)
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        match err.kind() {
            std::io::ErrorKind::NotFound => Self::NotFound { error_title: err.to_string() },
            std::io::ErrorKind::PermissionDenied => Self::PermissionDenied { error_title: err.to_string() },
            std::io::ErrorKind::AlreadyExists => Self::AlreadyExists { error_title: err.to_string() },
            _other => Self::IoError { reason: err.to_string() },
        }
    }
}

impl From<zip::result::ZipError> for Error {
    fn from(err: zip::result::ZipError) -> Self {
        use zip::result::ZipError;
        match err {
            ZipError::Io(io_err) => Self::from(io_err),
            ZipError::InvalidArchive(filename) => Self::InvalidZipArchive(filename),
            ZipError::FileNotFound => {
                Self::Custom {
                    reason: FinalError::with_title("Unexpected error in zip archive").detail("File not found"),
                }
            }
            ZipError::UnsupportedArchive(filename) => Self::UnsupportedZipArchive(filename),
        }
    }
}

impl From<ignore::Error> for Error {
    fn from(err: ignore::Error) -> Self {
        Self::DirWalkError { reason: err.to_string() }
    }
}

impl From<FinalError> for Error {
    fn from(err: FinalError) -> Self {
        Self::Custom { reason: err }
    }
}
