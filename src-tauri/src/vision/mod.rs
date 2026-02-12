//! Vision model support for multimodal AI processing.
//!
//! This module provides an interface for running vision-language models
//! like MiniCPM-V and Qwen2-VL on screenshots captured during voice input.
//!
//! # Architecture
//!
//! The vision module is designed with a trait-based interface to allow
//! different backends (ONNX Runtime, llama.cpp, external processes).
//!
//! # Current Status
//!
//! - Interface: Ready
//! - ONNX Runtime backend: Planned (MiniCPM-V has official ONNX export)
//! - llama.cpp backend: Blocked (llama-cpp-2 crate doesn't support vision)
//!
//! # Model Requirements
//!
//! For RTX 4060 (8GB VRAM), recommended models:
//! - MiniCPM-V 2.6 INT4 (~6-8GB VRAM)
//! - Qwen2-VL 7B INT4 (~6-8GB VRAM)

use std::path::Path;

/// Result type for vision operations.
pub type VisionResult<T> = Result<T, VisionError>;

/// Errors that can occur during vision processing.
#[derive(Debug, Clone)]
pub enum VisionError {
    /// Model not found at the specified path.
    ModelNotFound(String),
    /// Failed to load the model.
    LoadError(String),
    /// Failed to process the image.
    ProcessingError(String),
    /// Vision support not compiled in.
    NotSupported,
    /// Invalid input (e.g., corrupted image).
    InvalidInput(String),
}

impl std::fmt::Display for VisionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VisionError::ModelNotFound(p) => write!(f, "Model not found: {}", p),
            VisionError::LoadError(e) => write!(f, "Failed to load model: {}", e),
            VisionError::ProcessingError(e) => write!(f, "Processing failed: {}", e),
            VisionError::NotSupported => write!(f, "Vision support not available"),
            VisionError::InvalidInput(e) => write!(f, "Invalid input: {}", e),
        }
    }
}

impl std::error::Error for VisionError {}

/// A vision-language model engine.
///
/// Implementations handle loading a multimodal model and running inference
/// on images with text prompts.
pub trait VisionEngine: Send + Sync {
    /// Process an image with a text prompt.
    ///
    /// # Arguments
    ///
    /// * `image` - PNG-encoded image bytes
    /// * `prompt` - Text prompt describing what to extract from the image
    ///
    /// # Returns
    ///
    /// The model's text response.
    fn process(&self, image: &[u8], prompt: &str) -> VisionResult<String>;

    /// Check if the engine is ready to process requests.
    fn is_ready(&self) -> bool;
}

/// Configuration for vision model loading.
#[derive(Debug, Clone)]
pub struct VisionConfig {
    /// Path to the model directory or file.
    pub model_path: String,
    /// Maximum image dimension (larger images will be resized).
    pub max_image_size: u32,
    /// Number of threads for inference.
    pub num_threads: i32,
}

impl Default for VisionConfig {
    fn default() -> Self {
        Self {
            model_path: String::new(),
            max_image_size: 448,
            num_threads: 4,
        }
    }
}

/// Load a vision engine based on available backends.
///
/// # Current Implementation
///
/// Returns a stub implementation that logs warnings. Full implementation
/// requires ONNX Runtime backend (planned) or llama.cpp vision support.
///
/// # Arguments
///
/// * `config` - Configuration for the vision engine.
pub fn load_vision_engine(config: VisionConfig) -> VisionResult<Box<dyn VisionEngine>> {
    // Check if model path exists
    if !Path::new(&config.model_path).exists() {
        return Err(VisionError::ModelNotFound(config.model_path));
    }

    // For now, return a stub implementation
    // In the future, this will try ONNX Runtime backend first
    log::warn!(
        "Vision engine requested but not yet fully implemented. \
         Model path: {}",
        config.model_path
    );

    Ok(Box::new(StubVisionEngine))
}

/// A stub implementation that logs and returns errors.
///
/// This is used when vision support is not compiled in or when
/// the model fails to load.
pub struct StubVisionEngine;

impl VisionEngine for StubVisionEngine {
    fn process(&self, _image: &[u8], _prompt: &str) -> VisionResult<String> {
        Err(VisionError::NotSupported)
    }

    fn is_ready(&self) -> bool {
        false
    }
}

/// Preprocess an image for vision model input.
///
/// # Arguments
///
/// * `image` - PNG-encoded image bytes.
/// * `max_size` - Maximum dimension (width or height).
///
/// # Returns
///
/// Resized PNG bytes suitable for vision model input.
pub fn preprocess_image(image: &[u8], max_size: u32) -> VisionResult<Vec<u8>> {
    use image::io::Reader as ImageReader;
    use std::io::Cursor;

    // Decode PNG
    let reader = ImageReader::new(Cursor::new(image))
        .with_guessed_format()
        .map_err(|e| VisionError::InvalidInput(e.to_string()))?;

    let mut img = reader
        .decode()
        .map_err(|e| VisionError::InvalidInput(e.to_string()))?;

    // Resize if needed
    let (width, height) = (img.width(), img.height());
    if width > max_size || height > max_size {
        let ratio = (max_size as f32) / (width.max(height) as f32);
        let new_width = (width as f32 * ratio) as u32;
        let new_height = (height as f32 * ratio) as u32;

        img = img.resize(new_width, new_height, image::imageops::FilterType::Lanczos3);
    }

    // Encode back to PNG
    let mut output = Vec::new();
    let mut cursor = Cursor::new(&mut output);
    img.write_to(&mut cursor, image::ImageFormat::Png)
        .map_err(|e| VisionError::ProcessingError(e.to_string()))?;

    Ok(output)
}

/// Build a prompt for vision-based text polishing.
///
/// Combines the transcribed text with a request to use visual context.
pub fn build_vision_prompt(transcribed_text: &str, _lang: &str) -> String {
    format!(
        "The user said: \"{}\"\n\n\
        Look at the current screenshot and help complete or correct what the user is trying to do. \
        Consider the visual context (what app is open, what text is visible, etc.) \
        and provide the final text that should be inserted. \
        Output ONLY the corrected text, no explanations.",
        transcribed_text
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vision_error_display() {
        let err = VisionError::ModelNotFound("/path/to/model".to_string());
        assert!(err.to_string().contains("Model not found"));
    }

    #[test]
    fn test_build_vision_prompt() {
        let prompt = build_vision_prompt("Hello world", "en");
        assert!(prompt.contains("Hello world"));
        assert!(prompt.contains("screenshot"));
    }
}
