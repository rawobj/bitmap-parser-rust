mod bitmap;
mod error;
mod parser;

pub use bitmap::{BitmapFile, BitmapHeader, BitmapInfoHeader, RGBA};
pub use error::BitmapParseError;
