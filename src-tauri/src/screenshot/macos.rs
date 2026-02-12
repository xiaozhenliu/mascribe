//! macOS screenshot capture using CGWindow API.

use image::{ImageBuffer, Rgba};

/// Capture the main display.
pub fn capture_screen() -> Result<Vec<u8>, String> {
    use core_graphics::display::CGDisplay;

    let main_display = CGDisplay::main();
    let bounds = main_display.bounds();

    println!("[screenshot] Capturing main display: {:?}", bounds);

    // Capture the entire display
    let cg_image = CGDisplay::screenshot(
        bounds,
        0, // kCGWindowListOptionAll
        0, // kCGNullWindowID
        0, // image options
    ).ok_or("Failed to capture screen: screenshot returned None")?;

    // Convert to PNG bytes
    let png_bytes = cgimage_to_png(&cg_image)?;

    println!("[screenshot] Captured {} bytes", png_bytes.len());
    Ok(png_bytes)
}

/// Convert CGImage to PNG bytes.
fn cgimage_to_png(cg_image: &core_graphics::image::CGImage) -> Result<Vec<u8>, String> {
    let width = cg_image.width() as u32;
    let height = cg_image.height() as u32;

    // Get raw pixel data
    let data = cg_image.data();
    let bytes_per_row = cg_image.bytes_per_row();
    let data_slice: &[u8] = data.bytes();
    let data_ptr = data_slice.as_ptr();

    // Create image buffer
    let mut img_buffer = ImageBuffer::<Rgba<u8>, Vec<u8>>::new(width, height);

    for y in 0..height {
        for x in 0..width {
            let src_offset = (y as usize * bytes_per_row) + (x as usize * 4);
            unsafe {
                let r = *data_ptr.add(src_offset);
                let g = *data_ptr.add(src_offset + 1);
                let b = *data_ptr.add(src_offset + 2);
                let a = *data_ptr.add(src_offset + 3);
                img_buffer.put_pixel(x, y, Rgba([r, g, b, a]));
            }
        }
    }

    // Encode to PNG
    let mut output = Vec::new();
    let encoder = image::codecs::png::PngEncoder::new(&mut output);
    img_buffer
        .write_with_encoder(encoder)
        .map_err(|e| format!("Failed to encode PNG: {}", e))?;

    Ok(output)
}
