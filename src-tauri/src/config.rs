use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// Default polishing prompt — instructs LLM to clean up speech-to-text output.
/// Placeholders: {text} = input text, {lang} = detected language code (zh, en, ja, etc.)
pub const DEFAULT_POLISH_PROMPT: &str = "\
Language: {lang}
You are a speech-to-text post-processor. Your ONLY job is to clean up the speech transcript below.

Rules:
- Output ONLY the cleaned transcript, nothing else
- Remove filler words (嗯、呃、那个、就是、然后、啊、um、uh)
- Remove repeated words and false starts, but keep repeated meaning if wording differs
- Fix punctuation
- Do NOT translate: if the speaker mixes Chinese and English, keep both languages as spoken
- Use screen context (if provided) ONLY to fix homophones (同音字), e.g. deep sick → deepseek
- Do NOT include or summarize screen content in your output

{text}";

const SENSEVOICE_DIR: &str = "sensevoice/sherpa-onnx-sense-voice-zh-en-ja-ko-yue-int8-2024-07-17";
const POLISH_MODEL_FILE: &str = "qwen2.5-1.5b/qwen2.5-1.5b-instruct-q4_k_m.gguf";
const VISION_MODEL_DIR: &str = "minicpm-v-2_6";

fn models_root() -> PathBuf {
    dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("com.mascribe")
        .join("models")
}

#[derive(Serialize, Deserialize, Clone)]
pub struct AppConfig {
    pub model_dir: String,
    pub language: String,
    pub num_threads: i32,
    pub use_itn: bool,
    /// Path to the GGUF model file for local AI text polishing.
    pub polish_model_path: String,
    /// Enable AI text polishing after transcription.
    pub polish_enabled: bool,
    /// Polishing mode: "local" (GGUF model) or "api" (OpenAI-compatible).
    pub polish_mode: String,
    /// Directory to save WAV recordings.
    pub recordings_dir: String,
    /// Custom prompt template for AI polishing. Use {text} and {lang} as placeholders.
    pub polish_prompt: String,
    /// Global shortcut string, e.g. "Alt+Space".
    pub shortcut: String,
    /// Online API endpoint (OpenAI-compatible), e.g. "https://api.stepfun.com/v1/chat/completions".
    pub api_endpoint: String,
    /// API key for the online polishing service.
    pub api_key: String,
    /// Model name for the online API, e.g. "step-1-flash".
    pub api_model: String,
    /// Screenshot context mode: "disabled" | "save" | "api"
    pub screenshot_mode: String,
    /// Max screenshot dimension for resizing (0 = no limit)
    pub screenshot_max_size: u32,
    /// Vision model path for local multimodal processing (MiniCPM-V or Qwen2-VL)
    pub vision_model_path: String,
    /// Vision mode: "disabled" | "local" | "api"
    pub vision_mode: String,
    /// Max image dimension for vision model (0 = no limit)
    pub vision_max_image_size: u32,
    /// OCR API endpoint (e.g., Ollama at http://localhost:11434/v1)
    pub ocr_endpoint: String,
    /// OCR model name (e.g., "glm-ocr")
    pub ocr_model: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        let models_root = models_root();
        let model_dir = models_root.join(SENSEVOICE_DIR).to_string_lossy().to_string();
        let polish_model_path = models_root
            .join(POLISH_MODEL_FILE)
            .to_string_lossy()
            .to_string();
        let recordings_dir = dirs::data_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("com.mascribe")
            .join("recordings")
            .to_string_lossy()
            .to_string();
        Self {
            model_dir,
            language: "auto".to_string(),
            num_threads: 4,
            use_itn: true,
            polish_model_path,
            polish_enabled: true,
            polish_mode: "local".to_string(),
            recordings_dir,
            polish_prompt: DEFAULT_POLISH_PROMPT.to_string(),
            shortcut: "Alt+Space".to_string(),
            api_endpoint: String::new(),
            api_key: String::new(),
            api_model: String::new(),
            screenshot_mode: "disabled".to_string(),
            screenshot_max_size: 1024,
            vision_model_path: models_root.join(VISION_MODEL_DIR).to_string_lossy().to_string(),
            vision_mode: "disabled".to_string(),
            vision_max_image_size: 448,
            ocr_endpoint: "http://localhost:11434/v1".to_string(),
            ocr_model: "glm-ocr".to_string(),
        }
    }
}

impl AppConfig {
    /// Path to the persisted config JSON file.
    pub fn config_path() -> PathBuf {
        dirs::data_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("com.mascribe")
            .join("config.json")
    }

    /// Load config from disk. Missing fields are filled with defaults.
    pub fn load() -> Self {
        let path = Self::config_path();
        if !path.exists() {
            println!("[config] No config file found, using defaults");
            return Self::default();
        }
        match fs::read_to_string(&path) {
            Ok(contents) => {
                // Merge saved JSON on top of defaults so new fields get default values
                let defaults = Self::default();
                let mut value = serde_json::to_value(&defaults).unwrap();
                if let Ok(saved) = serde_json::from_str::<serde_json::Value>(&contents) {
                    if let (Some(base), Some(overlay)) = (value.as_object_mut(), saved.as_object())
                    {
                        for (k, v) in overlay {
                            base.insert(k.clone(), v.clone());
                        }
                    }
                }
                match serde_json::from_value(value) {
                    Ok(config) => {
                        println!("[config] Loaded from {}", path.display());
                        config
                    }
                    Err(e) => {
                        println!("[config] Parse error ({}), using defaults", e);
                        defaults
                    }
                }
            }
            Err(e) => {
                println!("[config] Read error ({}), using defaults", e);
                Self::default()
            }
        }
    }

    /// Persist config to disk.
    pub fn save(&self) -> Result<(), String> {
        let path = Self::config_path();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| format!("Failed to create config dir: {}", e))?;
        }
        let json =
            serde_json::to_string_pretty(self).map_err(|e| format!("Serialize error: {}", e))?;
        fs::write(&path, json).map_err(|e| format!("Write error: {}", e))?;
        println!("[config] Saved to {}", path.display());
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_has_valid_values() {
        let config = AppConfig::default();
        assert!(!config.language.is_empty());
        assert_eq!(config.language, "auto");
        assert!(config.num_threads > 0);
        assert_eq!(config.num_threads, 4);
        assert!(config.use_itn);
        assert!(config.polish_enabled);
        assert_eq!(config.polish_mode, "local");
        assert_eq!(config.shortcut, "Alt+Space");
        assert!(!config.polish_prompt.is_empty());
        assert_eq!(config.screenshot_mode, "disabled");
        assert_eq!(config.screenshot_max_size, 1024);
        assert_eq!(config.vision_mode, "disabled");
        assert_eq!(config.ocr_endpoint, "http://localhost:11434/v1");
        assert_eq!(config.ocr_model, "glm-ocr");
    }

    #[test]
    fn config_serialization_roundtrip() {
        let config = AppConfig::default();
        let json = serde_json::to_string(&config).unwrap();
        let restored: AppConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(config.language, restored.language);
        assert_eq!(config.num_threads, restored.num_threads);
        assert_eq!(config.shortcut, restored.shortcut);
        assert_eq!(config.polish_mode, restored.polish_mode);
        assert_eq!(config.api_endpoint, restored.api_endpoint);
        assert_eq!(config.screenshot_mode, restored.screenshot_mode);
        assert_eq!(config.ocr_model, restored.ocr_model);
    }

    #[test]
    fn config_load_with_missing_fields_uses_defaults() {
        let partial = r#"{"language": "en", "num_threads": 8}"#;
        let defaults = AppConfig::default();
        let mut value = serde_json::to_value(&defaults).unwrap();
        if let (Some(base), Some(overlay)) = (
            value.as_object_mut(),
            serde_json::from_str::<serde_json::Value>(partial)
                .unwrap()
                .as_object(),
        ) {
            for (k, v) in overlay {
                base.insert(k.clone(), v.clone());
            }
        }
        let config: AppConfig = serde_json::from_value(value).unwrap();
        // Saved values override defaults
        assert_eq!(config.language, "en");
        assert_eq!(config.num_threads, 8);
        // Other fields retain defaults
        assert_eq!(config.shortcut, "Alt+Space");
        assert!(config.polish_enabled);
        assert_eq!(config.ocr_model, "glm-ocr");
    }

    #[test]
    fn config_default_polish_prompt_contains_placeholders() {
        let config = AppConfig::default();
        assert!(config.polish_prompt.contains("{text}"));
        assert!(config.polish_prompt.contains("{lang}"));
    }

    #[test]
    fn config_empty_json_gets_all_defaults() {
        let empty = "{}";
        let defaults = AppConfig::default();
        let mut value = serde_json::to_value(&defaults).unwrap();
        if let (Some(base), Some(overlay)) = (
            value.as_object_mut(),
            serde_json::from_str::<serde_json::Value>(empty)
                .unwrap()
                .as_object(),
        ) {
            for (k, v) in overlay {
                base.insert(k.clone(), v.clone());
            }
        }
        let config: AppConfig = serde_json::from_value(value).unwrap();
        assert_eq!(config.language, defaults.language);
        assert_eq!(config.num_threads, defaults.num_threads);
        assert_eq!(config.shortcut, defaults.shortcut);
    }
}
