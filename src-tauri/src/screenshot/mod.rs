//! Cross-platform screenshot capture for active window context.
//!
//! On macOS: Uses CGWindow API
//! On Windows: Uses GDI (BitBlt) for compatibility

use std::path::PathBuf;

#[cfg(target_os = "macos")]
mod macos;

#[cfg(target_os = "windows")]
mod windows;

/// Capture the currently active screen area.
/// Returns PNG-encoded bytes.
pub fn capture_active_window() -> Result<Vec<u8>, String> {
    #[cfg(target_os = "macos")]
    return macos::capture_screen();

    #[cfg(target_os = "windows")]
    return windows::capture_screen();

    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    Err("Screenshot not supported on this platform".to_string())
}

/// Resize image to fit within max_dimension while maintaining aspect ratio.
pub fn resize_if_needed(png_bytes: Vec<u8>, max_dimension: u32) -> Result<Vec<u8>, String> {
    let img = image::load_from_memory(&png_bytes)
        .map_err(|e| format!("Failed to load image: {}", e))?;

    let (width, height) = (img.width(), img.height());

    // Check if resizing is needed
    if width <= max_dimension && height <= max_dimension {
        return Ok(png_bytes);
    }

    // Calculate new dimensions maintaining aspect ratio
    let ratio = if width > height {
        max_dimension as f32 / width as f32
    } else {
        max_dimension as f32 / height as f32
    };

    let new_width = (width as f32 * ratio) as u32;
    let new_height = (height as f32 * ratio) as u32;

    println!("[screenshot] Resizing from {}x{} to {}x{}", width, height, new_width, new_height);

    let resized = img.resize(new_width, new_height, image::imageops::FilterType::Lanczos3);

    // Encode back to PNG
    let mut output = Vec::new();
    let encoder = image::codecs::png::PngEncoder::new(&mut output);
    resized
        .write_with_encoder(encoder)
        .map_err(|e| format!("Failed to encode PNG: {}", e))?;

    Ok(output)
}

/// Encode PNG bytes to base64 string for API transmission.
pub fn encode_base64(png_bytes: &[u8]) -> String {
    use base64::{Engine as _, engine::general_purpose};
    general_purpose::STANDARD.encode(png_bytes)
}

/// Save screenshot to the app's data directory.
/// Returns the path where the screenshot was saved.
pub fn save_screenshot(png_bytes: &[u8]) -> Result<PathBuf, String> {
    let screenshots_dir = dirs::data_dir()
        .ok_or("Failed to get data directory")?
        .join("com.mac-voice-input")
        .join("screenshots");

    std::fs::create_dir_all(&screenshots_dir)
        .map_err(|e| format!("Failed to create screenshots directory: {}", e))?;

    let filename = format!(
        "screenshot-{}.png",
        chrono::Local::now().format("%Y%m%d-%H%M%S")
    );

    let path = screenshots_dir.join(&filename);

    std::fs::write(&path, png_bytes)
        .map_err(|e| format!("Failed to write screenshot: {}", e))?;

    println!("[screenshot] Saved to {}", path.display());
    Ok(path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_base64() {
        let data = b"hello world";
        let encoded = encode_base64(data);
        assert!(!encoded.is_empty());
        assert_eq!(encoded, "aGVsbG8gd29ybGQ=");
    }
}
