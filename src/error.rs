use std::{fmt::Display, io};

#[derive(Debug)]
pub enum BitmapParseError {
    IO(io::Error),
    InvalidSignature,
    UnexpectedEOF,
    UnsupportedHeader(u32),
    UnsupportedCompression(u32),
    UnsupportedBitDepth(u16),
    InvalidHeader(&'static str),
    InvalidPixelData,
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
            Self::UnsupportedHeader(size) => write!(f, "Unsupported DIB header size: {size}."),
            Self::UnsupportedCompression(value) => {
                write!(f, "Unsupported bitmap compression: {value}.")
            }
            Self::UnsupportedBitDepth(depth) => write!(f, "Unsupported bitmap bit depth: {depth}."),
            Self::InvalidHeader(message) => write!(f, "Invalid bitmap header: {message}."),
            Self::InvalidPixelData => write!(f, "Invalid bitmap pixel data."),
        }
    }
}

impl std::error::Error for BitmapParseError {}
