use std::path::Path;

use anyhow::{Context, Result, bail};

/// The only WAV format this app reads or writes.
pub const SPEC: hound::WavSpec = hound::WavSpec {
    channels: 1,
    sample_rate: 16_000,
    bits_per_sample: 32,
    sample_format: hound::SampleFormat::Float,
};

/// Loads a mono, 16 kHz, 32-bit float WAV file as f32 samples.
pub fn load(path: &Path) -> Result<Vec<f32>> {
    let mut reader = hound::WavReader::open(path).with_context(|| path.display().to_string())?;
    let spec = reader.spec();
    if spec != SPEC {
        bail!("{}: expected {SPEC:?}, got {spec:?}", path.display(),);
    }
    reader
        .samples::<f32>()
        .collect::<Result<Vec<_>, hound::Error>>()
        .with_context(|| path.display().to_string())
}

/// Writes mono 16 kHz f32 samples as a 32-bit float WAV file, overwriting any existing file.
pub fn write(path: &Path, samples: &[f32]) -> Result<()> {
    let mut writer = hound::WavWriter::create(path, SPEC)?;
    for &sample in samples {
        writer.write_sample(sample)?;
    }
    writer.finalize()?;
    Ok(())
}
