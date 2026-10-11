# Bitmap Parser

Bitmap decoder and command-line viewer implemented in Rust for reading Bitmap (BMP) images. It parses several BMP/DIB header variants, currently decodes uncompressed 24-bit BGR pixel data, and displays the preview.

## Why?
For learning purposes. 

**No any thirdparty library is used for any image parsing work, everything is fully self-coded for the sake of understanding every aspect of image parsing.**

The parser uses Rust's standard library for file I/O and decoding.
Cargo package `minifb` is used only by the viewer binary to paint the pixels for preview.

## Current support

The current implementation supports:

- BMP files with the `BM` signature
- OS/2 Core headers (12 bytes)
- Windows `BITMAPINFOHEADER` (40 bytes), V4 (108 bytes), and V5 (124 bytes) headers
- Uncompressed 24-bit BGR pixel data (`BI_RGB`)
- Positive-height bottom-up images
- Negative-height top-down images
- Four-byte row alignment and padding
- Flat row-major pixel storage as `Vec<u32>` using `0x00RRGGBB` pixels
- A simple preview window using `minifb`

The following features are not implemented:

- 1-bit, 4-bit, 8-bit, 16-bit, and 32-bit pixel formats
- Color palettes
- Channel masks and bitfields
- Alpha bitfields
- RLE4 and RLE8 compression.

The parser can identify several unsupported image types, but only RGB24 pixel data is currently decoded. Unsupported bit depths and compression modes return `BitmapDecoderError` rather than being decoded as some other format.


## Running the viewer

```text
cargo run --release --bin bitmap-parser -- sample/24bituncompressed.bmp
```

## Decoder benchmark

The benchmark compares the linear and SIMD RGB24 decoders. 
It reports average time per iteration, improvement over the linear decoder, percentage improvement, and a checksum confirming equivalent pixel output.

```text
cargo run --release --bin benchmark -- sample/24bituncompressed.bmp
BITMAP_BENCH_ITERATIONS=50 cargo run --release --bin benchmark -- sample/24bituncompressed.bmp
```

The viewer:

1. Opens and decodes the BMP
2. Uses the decoded `0x00RRGGBB` pixels in the format accepted by `minifb`
3. Uploads the display buffer once
4. Keeps the window open until it is closed or Esc is pressed

## Structure of Bitmap

A BMP file is composed of a file header, a DIB header, optional format-specific data, and the pixel array:

All multi-byte BMP fields are little-endian.

### File header

The current parser reads the 14-byte file header:

```rust
struct BitmapFileHeader {
    signature: [u8; 2], // 2 bytes, Offset `0000h`, 'BM'
    filesize: u32,      // 4 bytes, file size in bytes
    reserved: u32,      // 4 bytes, unused space
    offset: u32,        // 4 bytes,  offset to the beginning of bitmap data
}
```

### DIB headers

The implementation recognizes 12-byte OS/2 Core, 40-byte Windows `BITMAPINFOHEADER`, 108-byte V4, and 124-byte V5 headers. 
The fields `BITMAPINFOHEADER` are:

```rust
struct BitmapInfoHeader {
    header_size: u32,           // 4 bytes, size of BitmapInfoHeader
    width: i32,                 // 4 bytes, horizontal width of bitmap in pixels
    height: i32,                // 4 bytes, vertical height of bitmap in pixels
    planes: u16,                // 2 bytes, number of planes, (=1)
    bit_depth: u16,             // 2 bytes, bits per pixel
    compression: Compression,   // 4 bytes, Type of Compression, [0=BI_RGB, 1=BI_RLE8, 2=BI_RLE4]
    image_size: u32,            // 4 bytes, size of image (compressed) in bytes
    pix_per_meter_x: i32,       // 4 bytes, horizontal resolution
    pix_per_meter_y: i32,       // 4 bytes, vertical resolution
    color_count: u32,           // 4 bytes, number of actually used colors
    imp_color_count: u32,       // 4 bytes, number of important colors, 0=all
}
```

Windows INFO-family headers add compression, image size, resolution, and color-count fields to the Core headers. 
V4 and V5 headers add their corresponding color-space, mask, gamma, and profile fields.

## 24-bit pixel layout

A 24-bit BMP stores each pixel as three bytes in blue-green-red order:

```text
file byte order:  B  G  R
decoded value:    0x00RRGGBB
```

Each scanline is padded to a multiple of four bytes. 
For an image width of `width`, the unpadded pixel bytes and aligned row size are:

```text
pixel_bytes = width * 3
row_size    = (pixel_bytes + 3) & !3
```

Padding bytes are consumed as part of the row but are never interpreted as pixels.

## Row orientation

BMP height uses its sign to describe row order:

- Positive height: pixel rows are stored bottom-to-top
- Negative height: pixel rows are stored top-to-bottom

The decoder writes every source row to its final destination row. As a result,
`DecodedImage::pixels` is always top-to-bottom regardless of the source
orientation.

## References

- https://learn.microsoft.com/en-us/windows/win32/api/wingdi/ns-wingdi-bitmapcoreheader

- https://learn.microsoft.com/en-us/windows/win32/api/wingdi/ns-wingdi-bitmapinfoheader

- https://learn.microsoft.com/en-us/windows/win32/api/wingdi/ns-wingdi-bitmapv4header

- https://learn.microsoft.com/en-us/windows/win32/api/wingdi/ns-wingdi-bitmapv5header

- https://en.wikipedia.org/wiki/BMP_file_format

- https://gibberlings3.github.io/iesdp/file_formats/ie_formats/bmp.htm