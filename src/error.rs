use std::{fmt::Display, io};

use crate::reader;

#[derive(Debug)]
pub enum BitmapDecoderError {
    IO(io::Error),
    BufferError(reader::BufferError),
    InvalidSignature,
    UnsupportedInfoHeader(u32),
    InvalidHeader(&'static str),
    UnsupportedCompression(u32),
    UnsupportedBitDepth(u16),
    InvalidCompressionForDepth(u16),
    NotImplemented(crate::file::ImageType),
    InvalidPixelData,
}

impl From<io::Error> for BitmapDecoderError {
    fn from(value: io::Error) -> Self {
        BitmapDecoderError::IO(value)
    }
}

impl From<reader::BufferError> for BitmapDecoderError {
    fn from(value: reader::BufferError) -> Self {
        BitmapDecoderError::BufferError(value)
    }
}

impl Display for BitmapDecoderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnsupportedInfoHeader(size) => {
                write!(f, "Unsupported info header type. Got size: {}", size,)
            }
            Self::BufferError(err) => err.fmt(f),
            Self::IO(err) => write!(f, "Bitmap I/O error: {}", err),
            Self::InvalidSignature => {
                write!(f, "Invalid file signature. Not a valid bitmap file.",)
            }
            Self::InvalidHeader(message) => write!(f, "Invalid bitmap header: {message}."),
            Self::UnsupportedCompression(value) => {
                write!(f, "Unsupported bitmap compression: {value}.")
            }
            Self::UnsupportedBitDepth(depth) => write!(f, "Unsupported bitmap bit depth: {depth}."),
            Self::InvalidCompressionForDepth(depth) => {
                write!(f, "Invalid bitmap compression for bit depth: {depth}.")
            }
            Self::NotImplemented(image_type) => {
                write!(f, "Bitmap image type not implemented: {image_type}.")
            }
            Self::InvalidPixelData => write!(f, "Invalid bitmap pixel data."),
        }
    }
}

impl std::error::Error for BitmapDecoderError {}
