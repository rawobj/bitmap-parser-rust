use crate::{error::BitmapParseError, parser::BoxedBufferParser};

#[derive(Debug)]
pub struct BitmapFile {
    header: BitmapHeader,
    info_header: BitmapInfoHeader,
    // color_table: BitmapColorTable,
    // pixel_data: BitmapPixelData,
}

#[derive(Debug)]
// 14 bytes, BITMAPFILEHEADER
pub struct BitmapHeader {
    signature: [u8; 2], // 2 bytes, Offset `0000h`, 'BM'
    filesize: u32,      // 4 bytes, file size in bytes
    reserved: u32,      // 4 bytes, unused space
    offset: u32,        // 4 bytes,  offset to the beginning of bitmap data
}

#[derive(Debug)]
// 40 bytes, DIB BITMAPINFOHEADER, could be 108 (v4) or 124 (v5)
pub struct BitmapInfoHeader {
    header_size: u32,     // 4 bytes, size of BitmapInfoHeader
    pub width: u32,       // 4 bytes, horizontal width of bitmap in pixels
    pub height: u32,      // 4 bytes, vertical height of bitmap in pixels
    planes: u16,          // 2 bytes, number of planes, (=1)
    bit_depth: u16,       // 2 bytes, bits per pixel sued to store palette info [1, 4, 8, 16, 24]
    compression: u32,     // 4 bytes, Type of Compression, [0=BI_RGB, 1=BI_RLE8, 2=BI_RLE4]
    image_size: u32,      // 4 bytes, size of image (compressed) in bytes
    pix_per_meter_x: u32, // 4 bytes, horizontal resolution
    pix_per_meter_y: u32, // 4 bytes, vertical resolution
    color_count: u32,     // 4 bytes, number of actually used colors
    imp_color_count: u32, // 4 bytes, number of important colors, 0=all
}

#[derive(Debug)]
// Additional 68 bytes, for Windows BITMAPV4HEADER extension fields
pub struct BitmapV4Header {
    red_mask: u32,
    green_mask: u32,
    blue_mask: u32,
    alpha_mask: u32,
    color_space: u32,
    endpoints: [u32; 9],
    gamma_red: u32,
    gamma_green: u32,
    gamma_blue: u32,
}

#[derive(Debug)]
// Additional 16 bytes, for Windows BITMAPV5HEADER extension fields
pub struct BitmapV5Header {
    intent: u32,
    profile_data: u32,
    profile_size: u32,
    reserved: u32,
}

struct PixelData {}

impl BitmapFile {
    pub fn open(path: &str) -> Result<BitmapFile, BitmapParseError> {
        let data = std::fs::read(path)?.into_boxed_slice();

        // first two bytes of the data shall be the bitmap signature "BM"
        let signature = &data[0..2];
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

        // There could be 4 types of headers based on header size
        // 40 = BITMAPINFO HEADER, 108 = BITMAPV4HEADER, 124 = BITMAPV5HEADER

        let info_header = BitmapInfoHeader {
            header_size: parser.read_u32_le()?,
            width: parser.read_u32_le()?,
            height: parser.read_u32_le()?,
            planes: parser.read_u16_le()?,
            bit_depth: parser.read_u16_le()?,
            compression: parser.read_u32_le()?,
            image_size: parser.read_u32_le()?,
            pix_per_meter_x: parser.read_u32_le()?,
            pix_per_meter_y: parser.read_u32_le()?,
            color_count: parser.read_u32_le()?,
            imp_color_count: parser.read_u32_le()?,
        };

        let offset_bytes = parser.read_bytes(84)?;

        let mut color_table_size = 0;

        if info_header.bit_depth == 1 {
            color_table_size = 2;
        } else if info_header.bit_depth == 4 {
            color_table_size = 16;
        } else if info_header.bit_depth == 8 {
            color_table_size = 256;
        }

        let bitmap = BitmapFile {
            header,
            info_header,
        };

        dbg!(&bitmap);

        Ok(bitmap)
    }
}
