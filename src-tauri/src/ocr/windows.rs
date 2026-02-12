//! Windows native OCR using Windows.Media.Ocr (WinRT).
//!
//! Uses the built-in OcrEngine available since Windows 10 1507.
//! Same engine as PowerToys Text Extractor.
//!
//! The zh-Hans language engine natively handles mixed Chinese/English/code text —
//! no need to run separate engines for different languages.
//!
//! Requires OCR language pack installed (English usually pre-installed;
//! Chinese Simplified: Settings → Language & Region, or PowerShell:
//!   Add-WindowsCapability -Online -Name Language.OCR~~~zh-Hans~0.0.1.0)

use windows::core::HSTRING;
use windows::Globalization::Language;
use windows::Graphics::Imaging::BitmapDecoder;
use windows::Media::Ocr::OcrEngine;
use windows::Storage::Streams::{
    DataWriter, IOutputStream, IRandomAccessStream, InMemoryRandomAccessStream,
};
use windows::Win32::System::Com::{CoInitializeEx, COINIT_MULTITHREADED};

/// Recognize text in a PNG image using Windows.Media.Ocr.
///
/// Tries zh-Hans (Chinese Simplified) first — this engine handles ASCII,
/// English, numbers, and symbols natively, so mixed Chinese/English code
/// screenshots work with a single engine call.
///
/// Falls back to user profile languages if zh-Hans is not installed.
pub fn recognize_text(png_bytes: &[u8]) -> Result<String, String> {
    recognize_text_inner(png_bytes).map_err(|e| format!("Windows OCR failed: {}", e))
}

fn recognize_text_inner(png_bytes: &[u8]) -> Result<String, windows::core::Error> {
    // Ensure COM/WinRT is initialized on this thread (Tauri command handlers
    // run on a thread pool, not the main STA thread).
    // S_FALSE (already initialized) is silently ignored by .ok().
    unsafe {
        CoInitializeEx(None, COINIT_MULTITHREADED).ok()?;
    }

    let start = std::time::Instant::now();

    // 1. Write PNG bytes into an in-memory stream
    let stream = InMemoryRandomAccessStream::new()?;
    {
        let output: IOutputStream = stream.cast()?;
        let writer = DataWriter::CreateDataWriter(&output)?;
        writer.WriteBytes(png_bytes)?;
        writer.StoreAsync()?.get()?;
        writer.FlushAsync()?.get()?;
        writer.DetachStream()?;
    }

    // Reset stream to beginning for BitmapDecoder
    stream.Seek(0)?;

    // 2. Decode PNG into SoftwareBitmap
    let random_access: IRandomAccessStream = stream.cast()?;
    let decoder = BitmapDecoder::CreateAsync(&random_access)?.get()?;
    let bitmap = decoder.GetSoftwareBitmapAsync()?.get()?;

    // 3. Create OcrEngine — try zh-Hans first, fall back to user profile
    let engine = create_ocr_engine()?;

    // 4. Perform OCR (blocking — we're on a background thread)
    let result = engine.RecognizeAsync(&bitmap)?.get()?;

    // 5. Extract text from recognized lines
    let lines = result.Lines()?;
    let count = lines.Size()?;
    let mut text_lines = Vec::with_capacity(count as usize);

    for i in 0..count {
        let line = lines.GetAt(i)?;
        let text = line.Text()?.to_string();
        if !text.is_empty() {
            text_lines.push(text);
        }
    }

    let joined = text_lines.join("\n");
    println!(
        "[ocr:native] Windows OCR: {} lines, {} chars ({:.1}s)",
        count,
        joined.len(),
        start.elapsed().as_secs_f64()
    );

    Ok(joined)
}

/// Create an OcrEngine with the best available language.
///
/// Priority: zh-Hans → en-US → user profile languages.
/// zh-Hans handles mixed CJK + ASCII text natively.
fn create_ocr_engine() -> Result<OcrEngine, windows::core::Error> {
    // Try Chinese Simplified first (handles English/code natively)
    let zh_hans = Language::CreateLanguage(&HSTRING::from("zh-Hans"))?;
    if OcrEngine::IsLanguageSupported(&zh_hans)? {
        println!("[ocr:native] using zh-Hans OCR engine");
        return OcrEngine::TryCreateFromLanguage(&zh_hans);
    }

    // Try English as second choice
    let en_us = Language::CreateLanguage(&HSTRING::from("en-US"))?;
    if OcrEngine::IsLanguageSupported(&en_us)? {
        println!("[ocr:native] zh-Hans not available, falling back to en-US");
        return OcrEngine::TryCreateFromLanguage(&en_us);
    }

    // Last resort: whatever the user has
    println!("[ocr:native] falling back to user profile languages");
    OcrEngine::TryCreateFromUserProfileLanguages()
}
