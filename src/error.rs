use std::fmt;
use std::io;

pub type BARResult<T> = std::result::Result<T, BARFileError>;

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum BARFileError {
    InvalidFileFormat(String),
    CompressionError(String),
    IOError(String),
}
impl fmt::Display for BARFileError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BARFileError::InvalidFileFormat(msg) => {
                write!(f, "BARFile Error: Invalid File Format: {}", msg)
            }
            BARFileError::CompressionError(err) => write!(f, "BARFile Error: {}", err),
            BARFileError::IOError(msg) => write!(f, "BARFile Error: {}", msg),
        }
    }
}
impl std::error::Error for BARFileError {}

impl From<io::Error> for BARFileError {
    fn from(value: io::Error) -> Self {
        BARFileError::IOError(value.to_string())
    }
}
