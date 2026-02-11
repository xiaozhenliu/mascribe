use std::sync::Mutex;

use crate::audio::capture::AudioBuffer;
use crate::correction::dictionary::CorrectionDictionary;
use crate::polishing::engine::PolishingEngine;
use crate::recognition::engine::RecognitionEngine;

pub struct AppState {
    pub recognition_engine: Mutex<RecognitionEngine>,
    pub audio_buffer: AudioBuffer,
    pub correction_dict: CorrectionDictionary,
    pub last_result: Mutex<String>,
    /// Optional AI text polishing engine (None if model not available).
    pub polishing_engine: Option<Mutex<PolishingEngine>>,
}
