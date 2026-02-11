use hound::{WavSpec, WavWriter, SampleFormat};
use std::path::{Path, PathBuf};

/// Save raw f32 PCM samples to a WAV file.
/// Returns the path to the saved file.
pub fn save_wav(samples: &[f32], sample_rate: u32, dir: &Path) -> anyhow::Result<PathBuf> {
    std::fs::create_dir_all(dir)?;

    let timestamp = chrono::Local::now().format("%Y%m%d-%H%M%S");
    let filename = format!("recording-{}.wav", timestamp);
    let path = dir.join(&filename);

    let spec = WavSpec {
        channels: 1,
        sample_rate,
        bits_per_sample: 32,
        sample_format: SampleFormat::Float,
    };

    let mut writer = WavWriter::create(&path, spec)?;
    for &sample in samples {
        writer.write_sample(sample)?;
    }
    writer.finalize()?;

    println!("[wav_save] saved {} samples ({}Hz) to {}", samples.len(), sample_rate, path.display());
    Ok(path)
}
