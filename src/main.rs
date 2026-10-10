use bitmap_parser::BitmapFile;
use minifb::{Key, Window, WindowOptions};
use std::{env, error::Error, process};

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
    let decoded_image = bitmap.into_image_info()?.get_decoded_image()?;

    preview_image(
        decoded_image.width as usize,
        decoded_image.height as usize,
        decoded_image.pixels,
    );

    Ok(())
}

fn preview_image(width: usize, height: usize, pixels: Vec<u32>) {
    // Create the window
    let mut window = Window::new(
        "Bitmap Parser Preview",
        width,
        height,
        WindowOptions::default(),
    )
    .unwrap();

    window.update_with_buffer(&pixels, width, height).unwrap();

    // Keep the window open until the user presses Escape.
    while window.is_open() && !window.is_key_down(Key::Escape) {
        window.update();
    }
}
