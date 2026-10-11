use crate::{
    error::BitmapDecoderError,
    file::{DecodedImage, ImageInfo, ImageType},
};

pub fn decode(image_info: &ImageInfo) -> Result<DecodedImage, BitmapDecoderError> {
    // simd decoding only for RGB24
    if image_info.image_type != ImageType::RGB24 {
        return image_info.get_decoded_image();
    }

    let pixel_data = image_info
        .data
        .get(image_info.pixel_offset as usize..)
        .ok_or(BitmapDecoderError::InvalidPixelData)?;

    let width = image_info.width;
    let height = image_info.height;

    let pixel_bytes = width * 3; // 3 bytes per pixel for RGB24
    let row_size = (pixel_bytes + 3) & !3; // padded to 4 bytes

    let mut pixels = Vec::with_capacity((width * height) as usize);

    for row in 0..height {
        let row_index = if image_info.top_down {
            row
        } else {
            height - 1 - row
        };
        let start = (row_index * row_size) as usize;
        let end = start + pixel_bytes as usize;
        let row_data = pixel_data
            .get(start..end)
            .ok_or(BitmapDecoderError::InvalidPixelData)?;

        // now we use simd based row extraction
        let mut row_pixels = vec![0; width as usize];
        simd_row(row_data, &mut row_pixels);
        pixels.extend(row_pixels);
    }

    Ok(DecodedImage {
        width,
        height,
        pixels,
    })
}

fn simd_row(row: &[u8], destination: &mut [u32]) {
    #[cfg(target_arch = "x86_64")]
    {
        if std::is_x86_feature_detected!("ssse3") {
            unsafe { convert_row_ssse3(row, destination) };
            return;
        }
    }
    // fallback to scalar conversion
    convert_row_scalar(row, destination);
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "ssse3")]
unsafe fn convert_row_ssse3(row: &[u8], destination: &mut [u32]) {
    use std::arch::x86_64::*;
    unsafe {
        // output byte i = input byte mask[i], and a mask byte with the high bit set (-1) gives 0
        // so every group takes the B G R bytes of one pixel and adds a zero byte after them
        // `setr` takes the bytes in memory order, `set` takes them reversed
        // the mask is built once, outside the loop, and stays in an XMM register
        // XMM = one of the 16 SIMD registers of the CPU, each 128 bits (16 bytes) wide
        let shuffle = _mm_setr_epi8(
            0, 1, 2, -1, // pixel 0: B G R + zero
            3, 4, 5, -1, // pixel 1
            6, 7, 8, -1, // pixel 2
            9, 10, 11, -1, // pixel 3
        );

        let mut index = 0;

        // 4 pixels are 12 bytes but the load below always reads 16 bytes
        // so stop while 16 bytes are still left in the row, and 4 slots in the destination
        while index * 3 + 16 <= row.len() && index + 4 <= destination.len() {
            // compiles to one `movdqu xmm, [address]` (Move Double Quadword Unaligned)
            // the CPU reads 16 bytes from memory into the register in a single instruction
            // data comes from the L1 cache if it is there, otherwise from L2/L3/RAM, the hardware
            // prefetcher spots the sequential reads and fetches the next cache lines ahead of time
            // a cache line is 64 bytes, so one load covers a quarter of it, a load that crosses
            // a line boundary is a little slower, that's the price of not requiring alignment
            // register now holds: B0 G0 R0 B1 G1 R1 .. B3 G3 R3 + 4 unused bytes
            let input = _mm_loadu_si128(row.as_ptr().add(index * 3).cast());

            // compiles to one `pshufb xmm, xmm` (Packed Shuffle Bytes), register to register,
            // so no memory is touched, the vector unit reads the mask and writes all 16 output
            // bytes in parallel, the scalar version needs several instructions per pixel
            // for the same job (load 3 bytes, shift, or, store)
            // register now holds: B0 G0 R0 00 B1 G1 R1 00 .. B3 G3 R3 00
            // read as little endian u32 each group is already 0x00RRGGBB, so no shifting is needed
            let values = _mm_shuffle_epi8(input, shuffle);

            // compiles to one `movdqu [address], xmm`, 16 bytes go to the u32 buffer in one store
            // the store first lands in the CPU's store buffer and is written to the cache after,
            // so the CPU doesn't wait for it and continues with the next iteration
            // on a fresh output buffer the cache line has to be fetched from RAM before it can be
            // written, which is the memory traffic and page fault cost that limits large images
            _mm_storeu_si128(destination.as_mut_ptr().add(index).cast(), values);

            index += 4;
        }

        // the last few pixels that don't fill a whole step
        convert_row_scalar(&row[index * 3..], &mut destination[index..]);
    }
}

fn convert_row_scalar(row: &[u8], destination: &mut [u32]) {
    for (pixel, chunk) in destination.iter_mut().zip(row.chunks_exact(3)) {
        *pixel = pack_rgb24(chunk[2], chunk[1], chunk[0]);
    }
}

fn pack_rgb24(red: u8, green: u8, blue: u8) -> u32 {
    ((red as u32) << 16) | ((green as u32) << 8) | blue as u32
}
