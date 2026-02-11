use sherpa_rs::sense_voice::{SenseVoiceConfig, SenseVoiceRecognizer};
use std::path::Path;

pub struct RecognitionEngine {
    recognizer: SenseVoiceRecognizer,
}

impl RecognitionEngine {
    pub fn new(model_dir: &str, language: &str, num_threads: i32, use_itn: bool) -> anyhow::Result<Self> {
        let model_path = Path::new(model_dir).join("model.int8.onnx");
        let tokens_path = Path::new(model_dir).join("tokens.txt");

        if !model_path.exists() {
            anyhow::bail!("Model not found: {}", model_path.display());
        }
        if !tokens_path.exists() {
            anyhow::bail!("Tokens not found: {}", tokens_path.display());
        }

        let config = SenseVoiceConfig {
            model: model_path.to_string_lossy().to_string(),
            tokens: tokens_path.to_string_lossy().to_string(),
            language: language.to_string(),
            use_itn,
            provider: None,
            num_threads: Some(num_threads),
            debug: false,
        };

        let recognizer = SenseVoiceRecognizer::new(config)
            .map_err(|e| anyhow::anyhow!("Failed to initialize SenseVoice: {}", e))?;
        println!("SenseVoice model loaded from: {}", model_dir);

        Ok(Self { recognizer })
    }

    pub fn transcribe(&mut self, sample_rate: u32, samples: &[f32]) -> anyhow::Result<(String, String)> {
        let result = self.recognizer.transcribe(sample_rate, samples);
        Ok((result.text, result.lang))
    }
}
