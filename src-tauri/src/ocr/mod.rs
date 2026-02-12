//! Cross-platform OCR text recognition.
//!
//! On macOS: Uses Vision framework (VNRecognizeTextRequest) via the Neural Engine.
//! On Windows: Not supported (returns error).

#[cfg(target_os = "macos")]
mod macos;

/// Recognize text from a PNG screenshot using native OS OCR.
/// Returns the recognized text (lines joined by newline).
pub fn recognize_text(png_bytes: &[u8]) -> Result<String, String> {
    #[cfg(target_os = "macos")]
    return macos::recognize_text(png_bytes);

    #[cfg(target_os = "windows")]
    return Err("Native OCR not supported on Windows".to_string());

    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    Err("Native OCR not supported on this platform".to_string())
}
