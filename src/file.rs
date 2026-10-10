use std::{fmt, path::Path};

use crate::{
    error::BitmapDecoderError,
    reader::{BufferError, BufferReader},
};

/// The maximum width/height the decoder will process.
const MAX_WIDTH_HEIGHT: i32 = 0xFFFF;

// Bitmap File Header
#[derive(Debug)]
struct FileHeader {
    signature: [u8; 2], // 2 bytes, Offset `0000h`, 'BM'
    filesize: u32,      // 4 bytes, file size in bytes
    reserved: u32,      // 4 bytes, unused space
    offset: u32,        // 4 bytes,  offset to the beginning of bitmap data
}

// Bitmap header types and their respective sizes in bytes
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum HeaderType {
    Core, //12
    Info, // 40
    V4,   // 108
    V5,   // 124
}

impl TryFrom<u32> for HeaderType {
    type Error = BitmapDecoderError;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        match value {
            12 => Ok(Self::Core),
            40 => Ok(Self::Info),
            108 => Ok(Self::V4),
            124 => Ok(Self::V5),
            val => Err(BitmapDecoderError::UnsupportedInfoHeader(val)),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Compression {
    Rgb,            // 0 BI_RGB
    Rle8,           // 1
    Rle4,           // 2
    Bitfields,      // 3 BI_BITFIELDS
    AlphaBitfields, // 6
    Other(u32),
}

impl From<u32> for Compression {
    fn from(v: u32) -> Self {
        match v {
            0 => Self::Rgb,
            1 => Self::Rle8,
            2 => Self::Rle4,
            3 => Self::Bitfields,
            6 => Self::AlphaBitfields,
            other => Self::Other(other),
        }
    }
}

// Complete Info Header
#[derive(Debug)]
struct CoreHeader {
    // https://learn.microsoft.com/en-us/windows/win32/api/wingdi/ns-wingdi-bitmapcoreheader
    header_size: u32,
    width: i32,
    height: i32,
    planes: u16,
    bit_depth: u16,
}

impl CoreHeader {
    // The 12-byte OS/2 header stores width/height as u16
    // others use i32
    fn read(r: &mut BufferReader, header_size: u32, os2: bool) -> Result<Self, BitmapDecoderError> {
        let (width, height, planes, bit_depth) = if os2 {
            (
                r.read_u16_le()? as i32,
                r.read_u16_le()? as i32,
                r.read_u16_le()?,
                r.read_u16_le()?,
            )
        } else {
            (
                r.read_i32_le()?,
                r.read_i32_le()?,
                r.read_u16_le()?,
                r.read_u16_le()?,
            )
        };

        Ok(Self {
            header_size,
            width,
            height,
            planes,
            bit_depth,
        })
    }
}

#[derive(Debug)]
struct InfoHeader {
    // https://learn.microsoft.com/en-us/windows/win32/api/wingdi/ns-wingdi-bitmapinfoheader
    compression: Compression,
    image_size: u32,
    pix_per_meter_x: i32,
    pix_per_meter_y: i32,
    color_count: u32,
    imp_color_count: u32,
}

impl InfoHeader {
    fn read(r: &mut BufferReader) -> Result<Self, BitmapDecoderError> {
        Ok(Self {
            compression: r.read_u32_le()?.into(),
            image_size: r.read_u32_le()?,
            pix_per_meter_x: r.read_i32_le()?,
            pix_per_meter_y: r.read_i32_le()?,
            color_count: r.read_u32_le()?,
            imp_color_count: r.read_u32_le()?,
        })
    }
}

#[derive(Debug)]
struct V4Header {
    // https://learn.microsoft.com/en-us/windows/win32/api/wingdi/ns-wingdi-bitmapv4header
    red_mask: u32,
    green_mask: u32,
    blue_mask: u32,
    alpha_mask: u32,
    color_space: u32, // TODO: enum for color space types
    // CIEXYZTRIPLE: x, y, z for each of red, green, blue (36 bytes).
    endpoints: [(u32, u32, u32); 3],
    gamma_red: u32,
    gamma_green: u32,
    gamma_blue: u32,
}

impl V4Header {
    fn read(r: &mut BufferReader) -> Result<Self, BitmapDecoderError> {
        Ok(Self {
            red_mask: r.read_u32_le()?,
            green_mask: r.read_u32_le()?,
            blue_mask: r.read_u32_le()?,
            alpha_mask: r.read_u32_le()?,
            color_space: r.read_u32_le()?,
            endpoints: [
                (r.read_u32_le()?, r.read_u32_le()?, r.read_u32_le()?),
                (r.read_u32_le()?, r.read_u32_le()?, r.read_u32_le()?),
                (r.read_u32_le()?, r.read_u32_le()?, r.read_u32_le()?),
            ],
            gamma_red: r.read_u32_le()?,
            gamma_green: r.read_u32_le()?,
            gamma_blue: r.read_u32_le()?,
        })
    }
}

#[derive(Debug)]
struct V5Header {
    // https://learn.microsoft.com/en-us/windows/win32/api/wingdi/ns-wingdi-bitmapv5header
    intent: u32,
    profile_data: u32,
    profile_size: u32,
    reserved: u32,
}

impl V5Header {
    fn read(r: &mut BufferReader) -> Result<Self, BitmapDecoderError> {
        Ok(Self {
            intent: r.read_u32_le()?,
            profile_data: r.read_u32_le()?,
            profile_size: r.read_u32_le()?,
            reserved: r.read_u32_le()?,
        })
    }
}

// Info Header Variants as per the header type
#[derive(Debug)]
pub enum BitmapInfoHeader {
    Core(CoreHeader),
    Info(CoreHeader, InfoHeader),
    V4(CoreHeader, InfoHeader, V4Header),
    V5(CoreHeader, InfoHeader, V4Header, V5Header),
}

impl BitmapInfoHeader {
    pub fn read(r: &mut BufferReader, header_size: u32) -> Result<Self, BitmapDecoderError> {
        let header_type = HeaderType::try_from(header_size)?;
        let core_header = CoreHeader::read(r, header_size, header_type == HeaderType::Core)?;

        Ok(match header_type {
            HeaderType::Core => Self::Core(core_header),
            HeaderType::Info => Self::Info(core_header, InfoHeader::read(r)?),
            HeaderType::V4 => Self::V4(core_header, InfoHeader::read(r)?, V4Header::read(r)?),
            HeaderType::V5 => Self::V5(
                core_header,
                InfoHeader::read(r)?,
                V4Header::read(r)?,
                V5Header::read(r)?,
            ),
        })
    }

    fn get_type(&self) -> HeaderType {
        match self {
            Self::Core(..) => HeaderType::Core,
            Self::Info(..) => HeaderType::Info,
            Self::V4(..) => HeaderType::V4,
            Self::V5(..) => HeaderType::V5,
        }
    }

    pub fn values(
        &self,
    ) -> (
        &CoreHeader,
        Option<&InfoHeader>,
        Option<&V4Header>,
        Option<&V5Header>,
    ) {
        match self {
            Self::Core(c) => (c, None, None, None),
            Self::Info(c, i) => (c, Some(i), None, None),
            Self::V4(c, i, v4) => (c, Some(i), Some(v4), None),
            Self::V5(c, i, v4, v5) => (c, Some(i), Some(v4), Some(v5)),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum ImageType {
    Palette,                  // has palette
    RGB16,                    // has channel masks
    RGB24,                    // no palette, no channel masks
    RGB32,                    // no palette, no channel masks
    RLE8Compressed,           // has palette
    RLE4Compressed,           // has palette
    BitfieldsCompressed,      // has channel masks (16/32)
    AlphaBitfieldsCompressed, // has channel masks
}

impl ImageType {
    fn from_info_headers(
        core: &CoreHeader,
        info: Option<&InfoHeader>,
    ) -> Result<Self, BitmapDecoderError> {
        let Some(info) = info else {
            // Core headers do not contain compression information
            // they describe the original uncompressed, palette/RGB bitmap formats.
            return match core.bit_depth {
                1 | 4 | 8 => Ok(Self::Palette),
                24 => Ok(Self::RGB24),
                bit_depth => Err(BitmapDecoderError::UnsupportedBitDepth(bit_depth)),
            };
        };

        match (&info.compression, core.bit_depth) {
            (Compression::Rgb, 1 | 4 | 8) => Ok(Self::Palette),
            (Compression::Rgb, 16) => Ok(Self::RGB16),
            (Compression::Rgb, 24) => Ok(Self::RGB24),
            (Compression::Rgb, 32) => Ok(Self::RGB32),
            (Compression::Rle8, 8) => Ok(Self::RLE8Compressed),
            (Compression::Rle4, 4) => Ok(Self::RLE4Compressed),
            (Compression::Bitfields, 16 | 32) => Ok(Self::BitfieldsCompressed),
            (Compression::AlphaBitfields, 32) => Ok(Self::AlphaBitfieldsCompressed),
            (Compression::Other(o), _) => Err(BitmapDecoderError::UnsupportedCompression(*o)),
            (_, bit_depth) => Err(BitmapDecoderError::UnsupportedBitDepth(bit_depth)),
        }
    }
}

pub struct BitmapFile {
    file_header: FileHeader,
    headers: BitmapInfoHeader,
    header_end: usize, // where the header bytes end and a palette/pixeldata would start
    data: Vec<u8>,
}

// custom debug implementation to avoid printing the entire pixel data
impl fmt::Debug for BitmapFile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("BitmapFile")
            .field("file_header", &self.file_header)
            .field("headers", &self.headers)
            .field("header_end", &self.header_end)
            .field("data_len", &self.data.len())
            .finish()
    }
}

impl BitmapFile {
    pub fn open(path: impl AsRef<Path>) -> Result<BitmapFile, BitmapDecoderError> {
        let data = std::fs::read(path)?;

        // first two bytes of the data shall be the bitmap signature "BM"
        let signature = data.get(..2).ok_or(BufferError::UnexpectedEOF)?;
        if signature != b"BM" {
            return Err(BitmapDecoderError::InvalidSignature);
        }

        let mut r = BufferReader::new(&data);

        // File Header
        let file_header = FileHeader {
            signature: r.read_bytes(2)?.try_into().unwrap(),
            filesize: r.read_u32_le()?,
            reserved: r.read_u32_le()?,
            offset: r.read_u32_le()?,
        };

        // Info Header
        let header_size = r.read_u32_le()?;
        let info_header = BitmapInfoHeader::read(&mut r, header_size)?;

        // header validation
        let (core, ..) = info_header.values();
        if core.width < 1
            || core.width > MAX_WIDTH_HEIGHT
            || core.height == 0
            || core.height.unsigned_abs() > MAX_WIDTH_HEIGHT as u32
        {
            return Err(BitmapDecoderError::InvalidHeader(
                "Invalid image dimensions",
            ));
        }

        if core.planes != 1 {
            return Err(BitmapDecoderError::InvalidHeader("Planes must be one"));
        }

        let header_end = r.position();

        // pixel data must start after the headers and inside the file
        let offset = file_header.offset as usize;
        if offset < header_end || offset > data.len() {
            return Err(BitmapDecoderError::InvalidHeader(
                "Invalid pixel data offset",
            ));
        }
        println!("Header end: {header_end}, Pixel data offset: {offset}");

        Ok(BitmapFile {
            file_header,
            headers: info_header,
            header_end,
            data,
        })
    }

    fn into_image_info(self) -> Result<ImageInfo, BitmapDecoderError> {
        let (core, info, ..) = self.headers.values();

        // image type from compresion bitdepth and header type
        let image_type = ImageType::from_info_headers(core, info)?;

        let width = core.width as u32;
        let height = core.height.unsigned_abs();
        let bit_depth = core.bit_depth;

        // where the pixel data lives inside the file buffer
        let pixel_offset = self.file_header.offset;

        // generate the image info from the captured headers
        Ok(ImageInfo {
            width,
            height,
            top_down: core.height < 0,
            bit_depth,
            image_type,
            pixel_offset,
        })
    }
}


#[derive(Debug)]
pub struct ImageInfo {
    width: u32,
    height: u32, // taking always positive with top_down for orientation
    top_down: bool,
    bit_depth: u16,
    image_type: ImageType,
    pixel_offset: u32, // where the pixel data starts in the file buffer

    // color_mask: None,
    // color_palette: None,
    // icc_profile: None
}
