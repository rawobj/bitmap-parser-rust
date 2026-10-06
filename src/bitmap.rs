use crate::{error::BitmapParseError, parser::BoxedBufferParser};

#[derive(Debug)]
pub enum Compression {
    Rgb,
    Rle8,
    Rle4,
    Unknown(u32),
}

impl Compression {
    fn parse(value: u32) -> Self {
        match value {
            0 => Self::Rgb,
            1 => Self::Rle8,
            2 => Self::Rle4,
            value => Self::Unknown(value),
        }
    }
}

#[derive(Debug)]
pub struct BitmapFile {
    pub header: BitmapHeader,
    pub info_header: BitmapInfoHeader,
    // color_table: BitmapColorTable,
    pub pixels: Vec<RGBA>,
}

#[derive(Debug)]
// 14 bytes, BITMAPFILEHEADER
pub struct BitmapHeader {
    pub signature: [u8; 2], // 2 bytes, Offset `0000h`, 'BM'
    pub filesize: u32,      // 4 bytes, file size in bytes
    pub reserved: u32,      // 4 bytes, unused space
    pub offset: u32,        // 4 bytes,  offset to the beginning of bitmap data
}

#[derive(Debug)]
// 40 bytes, DIB BITMAPINFOHEADER, could be 108 (v4) or 124 (v5)
pub struct BitmapInfoHeader {
    pub header_size: u32,         // 4 bytes, size of BitmapInfoHeader
    pub width: i32,               // 4 bytes, horizontal width of bitmap in pixels
    pub height: i32,              // 4 bytes, vertical height of bitmap in pixels
    pub planes: u16,              // 2 bytes, number of planes, (=1)
    pub bit_depth: u16, // 2 bytes, bits per pixel sued to store palette info [1, 4, 8, 16, 24]
    pub compression: Compression, // 4 bytes, Type of Compression, [0=BI_RGB, 1=BI_RLE8, 2=BI_RLE4]
    pub image_size: u32, // 4 bytes, size of image (compressed) in bytes
    pub pix_per_meter_x: i32, // 4 bytes, horizontal resolution
    pub pix_per_meter_y: i32, // 4 bytes, vertical resolution
    pub color_count: u32, // 4 bytes, number of actually used colors
    pub imp_color_count: u32, // 4 bytes, number of important colors, 0=all
}

// #[derive(Debug)]
// // Additional 68 bytes, for Windows BITMAPV4HEADER extension fields
// pub struct BitmapV4Header {
//     red_mask: u32,
//     green_mask: u32,
//     blue_mask: u32,
//     alpha_mask: u32,
//     color_space: u32,
//     endpoints: [u32; 9],
//     gamma_red: u32,
//     gamma_green: u32,
//     gamma_blue: u32,
// }

// #[derive(Debug)]
// // Additional 16 bytes, for Windows BITMAPV5HEADER extension fields
// pub struct BitmapV5Header {
//     intent: u32,
//     profile_data: u32,
//     profile_size: u32,
//     reserved: u32,
// }

#[derive(Clone, Debug)]
pub struct RGBA(pub u8, pub u8, pub u8, pub u8);

impl RGBA {
    pub fn to_u32_rgb(&self) -> u32 {
        ((self.0 as u32) << 16) | ((self.1 as u32) << 8) | (self.2 as u32)
    }
}

impl BitmapFile {
    pub fn open(path: &str) -> Result<BitmapFile, BitmapParseError> {
        let data = std::fs::read(path)?;

        // first two bytes of the data shall be the bitmap signature "BM"
        let signature = data.get(..2).ok_or(BitmapParseError::UnexpectedEOF)?;
        if signature != b"BM" {
            return Err(BitmapParseError::InvalidSignature);
        }

        let mut parser = BoxedBufferParser::new(&data);

        // parsing work
        let header = BitmapHeader {
            signature: parser.read_bytes(2)?.try_into().unwrap(),
            filesize: parser.read_u32_le()?,
            reserved: parser.read_u32_le()?,
            offset: parser.read_u32_le()?,
        };

        let header_size = parser.read_u32_le()?;
        // There could be 4 types of headers based on header size
        // 40 = BITMAPINFO HEADER, 108 = BITMAPV4HEADER, 124 = BITMAPV5HEADER

        // Not gonna support BITMAPCOREHEADER (12)
        if header_size == 12 {
            return Err(BitmapParseError::UnsupportedHeader(header_size));
        }

        let info_header = BitmapInfoHeader {
            header_size,
            width: parser.read_i32_le()?,
            height: parser.read_i32_le()?,
            planes: parser.read_u16_le()?,
            bit_depth: parser.read_u16_le()?,
            compression: Compression::parse(parser.read_u32_le()?),
            image_size: parser.read_u32_le()?,
            pix_per_meter_x: parser.read_i32_le()?,
            pix_per_meter_y: parser.read_i32_le()?,
            color_count: parser.read_u32_le()?,
            imp_color_count: parser.read_u32_le()?,
        };

        if info_header.planes != 1 {
            return Err(BitmapParseError::InvalidHeader("Planes must be one"));
        }

        if info_header.width <= 0 || info_header.height == 0 {
            return Err(BitmapParseError::InvalidHeader("Invalid image dimensions"));
        }

        if info_header.image_size == 0 {
            return Err(BitmapParseError::InvalidPixelData);
        }

        // Only 24bit supported for now
        if info_header.bit_depth != 24 {
            return Err(BitmapParseError::UnsupportedBitDepth(info_header.bit_depth));
        }

        parser.seek(header.offset as usize)?;

        let pixels = match info_header.compression {
            Compression::Rgb => BitmapFile::decode_uncompressed(parser, &info_header)?,
            Compression::Rle4 => {
                return Err(BitmapParseError::UnsupportedCompression(1));
            }
            Compression::Rle8 => {
                return Err(BitmapParseError::UnsupportedCompression(2));
            }
            Compression::Unknown(value) => {
                return Err(BitmapParseError::UnsupportedCompression(value));
            }
        };

        let bitmap = BitmapFile {
            header,
            info_header,
            pixels,
        };

        Ok(bitmap)
    }

    fn decode_uncompressed(
        mut parser: BoxedBufferParser,
        info: &BitmapInfoHeader,
    ) -> Result<Vec<RGBA>, BitmapParseError> {
        let width = info.width as usize;
        let height = info.height.unsigned_abs() as usize;
        let pixel_bytes = width * 3;
        let row_size = (pixel_bytes + 3) & !3;
        let pixel_count = width * height;

        let mut pixels = vec![RGBA(0, 0, 0, 0); pixel_count];

        for file_row in 0..height {
            let pixel_line = parser.read_bytes(row_size)?;
            let destination_row = if info.height > 0 {
                height - 1 - file_row
            } else {
                file_row
            };
            let destination_start = destination_row * width;

            let (pixel_line, _) = pixel_line[..pixel_bytes].as_chunks::<3>();
            for (column, pixel) in pixel_line.iter().enumerate() {
                pixels[destination_start + column] = RGBA(pixel[2], pixel[1], pixel[0], 255);
            }
        }

        Ok(pixels)
    }
}
