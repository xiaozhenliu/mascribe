use std::sync::Mutex;

use crate::audio::capture::AudioBuffer;
use crate::config::AppConfig;
use crate::correction::dictionary::CorrectionDictionary;
use crate::hotkey::NativeHotkeyHandle;
use crate::polishing::engine::PolishingEngine;
use crate::recognition::engine::RecognitionEngine;

pub struct AppState {
    pub recognition_engine: Mutex<RecognitionEngine>,
    pub audio_buffer: AudioBuffer,
    pub correction_dict: Mutex<CorrectionDictionary>,
    pub last_result: Mutex<String>,
    /// Optional AI text polishing engine (None if model not available).
    pub polishing_engine: Option<Mutex<PolishingEngine>>,
    /// Persisted app configuration (editable via Settings window).
    pub config: Mutex<AppConfig>,
    /// Native CGEventTap hotkey handle (for keys not supported by global-shortcut).
    pub native_hotkey: Mutex<Option<NativeHotkeyHandle>>,
}
