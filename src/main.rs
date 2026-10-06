use minifb::{Key, Window, WindowOptions};
use std::{env, error::Error, process};

use bitmap_parser::{BitmapFile, RGBA};

fn main() {
    let mut args = env::args();
    args.next();

    let filename = args.next().unwrap_or_else(|| {
        eprintln!("Didn't get a filename.");
        process::exit(1);
    });

    println!("Parsing {}..", filename);

    if let Err(e) = run(filename) {
        eprintln!("Application error: {e}");
        process::exit(1);
    }
}

fn run(filename: String) -> Result<(), Box<dyn Error>> {
    let bitmap = BitmapFile::open(&filename)?;

    dbg!(&bitmap.info_header);
    preview_image(
        bitmap.info_header.width as usize,
        bitmap.info_header.height.unsigned_abs() as usize,
        bitmap.pixels,
    );

    Ok(())
}

fn preview_image(width: usize, height: usize, pixels: Vec<RGBA>) {
    let buffer: Vec<u32> = pixels.iter().map(RGBA::to_u32_rgb).collect();

    // Create the window
    let mut window = Window::new(
        "Bitmap Parser Preview",
        width,
        height,
        WindowOptions::default(),
    )
    .unwrap();

    window.update_with_buffer(&buffer, width, height).unwrap();

    // Keep the window open until the user presses Escape.
    while window.is_open() && !window.is_key_down(Key::Escape) {
        window.update();
    }
}
