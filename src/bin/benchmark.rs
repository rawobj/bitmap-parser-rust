use std::{
    env,
    hint::black_box,
    time::{Duration, Instant},
};

use bitmap_decoder::{BitmapFile, simd};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    if std::is_x86_feature_detected!("ssse3") {
        println!("SSSE3 detected, using SIMD decoding for RGB24.");
    } else {
        println!("SSSE3 not detected, falling back to scalar decoding for RGB24.");
    }

    let path = env::args().nth(1).unwrap_or_else(|| "sample.bmp".into());
    let iterations = env::var("BITMAP_BENCH_ITERATIONS")
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or(20)
        .max(1);
    let image = BitmapFile::open(&path)?.into_image_info()?;
    let image_dimensions = image.get_decoded_image()?;
    let pixel_count = image_dimensions.pixels.len();
    let strategies = [("linear", true), ("SIMD", false)];
    let mut results = Vec::new();
    for (name, linear) in strategies {
        let mut pixels = if linear {
            image.get_decoded_image()?.pixels
        } else {
            simd::decode(&image)?.pixels
        };
        let start = Instant::now();
        for _ in 0..iterations {
            pixels = if linear {
                image.get_decoded_image()?.pixels
            } else {
                simd::decode(&image)?.pixels
            };
            black_box(&pixels);
        }
        let checksum = pixels
            .iter()
            .fold(0u32, |sum, pixel| sum.wrapping_add(*pixel));
        results.push((name, start.elapsed() / iterations as u32, checksum));
    }
    let baseline = results[0].1;
    println!("Bitmap: {path} | iterations: {iterations}");
    println!("Output buffer: {pixel_count} pixels (reused for every iteration)");
    println!(
        "{:<20} {:>14} {:>14} {:>12}",
        "decoder", "time", "improvement", "faster"
    );
    for (name, duration, checksum) in results {
        let improvement = baseline.as_secs_f64() - duration.as_secs_f64();
        let percent = if baseline.is_zero() {
            0.0
        } else {
            improvement / baseline.as_secs_f64() * 100.0
        };
        println!(
            "{:<20} {:>14} {:>+13} {:>10.2}% (checksum {checksum:08x})",
            name,
            format_duration(duration),
            format_signed_duration(improvement),
            percent
        );
    }

    fn format_signed_duration(seconds: f64) -> String {
        let sign = if seconds < 0.0 { "-" } else { "+" };
        let seconds = seconds.abs();
        if seconds < 1.0 {
            format!("{sign}{:.3} ms", seconds * 1_000.0)
        } else {
            format!("{sign}{seconds:.3} s")
        }
    }
    Ok(())
}

fn format_duration(duration: Duration) -> String {
    if duration.as_secs_f64() < 1.0 {
        format!("{:.3} ms", duration.as_secs_f64() * 1_000.0)
    } else {
        format!("{:.3} s", duration.as_secs_f64())
    }
}
