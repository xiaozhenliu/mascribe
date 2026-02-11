use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct AppConfig {
    pub model_dir: String,
    pub language: String,
    pub num_threads: i32,
    pub use_itn: bool,
}

impl Default for AppConfig {
    fn default() -> Self {
        let model_dir = dirs::home_dir()
            .unwrap()
            .join(".openclaw/models/sensevoice/sherpa-onnx-sense-voice-zh-en-ja-ko-yue-int8-2024-07-17")
            .to_string_lossy()
            .to_string();
        Self {
            model_dir,
            language: "auto".to_string(),
            num_threads: 4,
            use_itn: true,
        }
    }
}
