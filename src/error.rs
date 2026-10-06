use std::{fmt::Display, io};

#[derive(Debug)]
pub enum BitmapParseError {
    IO(io::Error),
    InvalidSignature,
    UnexpectedEOF,
}

impl From<io::Error> for BitmapParseError {
    fn from(value: io::Error) -> Self {
        BitmapParseError::IO(value)
    }
}

impl Display for BitmapParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::IO(err) => write!(f, "Bitmap I/O error: {}", err),
            Self::InvalidSignature => {
                write!(f, "Invalid file signature. Not a valid bitmap file.",)
            }
            Self::UnexpectedEOF => {
                write!(f, "Unexpected end of file.",)
            }
        }
    }
}

impl std::error::Error for BitmapParseError {}
