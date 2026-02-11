use std::sync::Mutex;

use crate::audio::capture::AudioBuffer;
use crate::correction::dictionary::CorrectionDictionary;
use crate::recognition::engine::RecognitionEngine;

pub struct AppState {
    pub recognition_engine: Mutex<RecognitionEngine>,
    pub audio_buffer: AudioBuffer,
    pub correction_dict: CorrectionDictionary,
    pub last_result: Mutex<String>,
}
