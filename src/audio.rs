use std::path::Path;

use anyhow::{bail, Context, Result};

/// Loads a mono, 16 kHz, 16-bit WAV file as normalized f32 samples.
pub fn load_wav(path: &Path) -> Result<Vec<f32>> {
    let mut reader = hound::WavReader::open(path).with_context(|| path.display().to_string())?;
    let spec = reader.spec();
    if (spec.channels, spec.sample_rate, spec.bits_per_sample) != (1, 16_000, 16) {
        bail!(
            "{}: expected mono 16 kHz 16-bit, got {} channel(s) {} Hz {}-bit",
            path.display(),
            spec.channels,
            spec.sample_rate,
            spec.bits_per_sample,
        );
    }
    reader
        .samples::<i16>()
        .map(|sample| sample.map(|sample| sample as f32 / 32768.0))
        .collect::<Result<Vec<_>, hound::Error>>()
        .with_context(|| path.display().to_string())
}
