use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct AppConfig {
    pub model_dir: String,
    pub language: String,
    pub num_threads: i32,
    pub use_itn: bool,
    /// Path to the GGUF model file for AI text polishing (e.g. Gemma 3 1B).
    pub polish_model_path: String,
    /// Enable AI text polishing after transcription.
    pub polish_enabled: bool,
}

impl Default for AppConfig {
    fn default() -> Self {
        let home = dirs::home_dir().unwrap();
        let model_dir = home
            .join(".openclaw/models/sensevoice/sherpa-onnx-sense-voice-zh-en-ja-ko-yue-int8-2024-07-17")
            .to_string_lossy()
            .to_string();
        let polish_model_path = home
            .join(".openclaw/models/gemma3/gemma-3-1b-it-q4_0.gguf")
            .to_string_lossy()
            .to_string();
        Self {
            model_dir,
            language: "auto".to_string(),
            num_threads: 4,
            use_itn: true,
            polish_model_path,
            polish_enabled: true,
        }
    }
}
