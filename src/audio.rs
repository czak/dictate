use std::path::Path;

use anyhow::{Context, Result, bail};

pub const SAMPLE_RATE: u32 = 16_000;
pub const CHANNELS: u16 = 1;
pub const BITS_PER_SAMPLE: u16 = 16;

/// Loads a mono, 16 kHz, 16-bit WAV file as normalized f32 samples.
pub fn load_wav(path: &Path) -> Result<Vec<f32>> {
    let mut reader = hound::WavReader::open(path).with_context(|| path.display().to_string())?;
    let spec = reader.spec();
    if (spec.channels, spec.sample_rate, spec.bits_per_sample)
        != (CHANNELS, SAMPLE_RATE, BITS_PER_SAMPLE)
    {
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

/// Writes mono 16 kHz 16-bit PCM samples as a WAV file, overwriting any existing file.
pub fn write_wav(path: &Path, samples: &[i16]) -> Result<()> {
    let spec = hound::WavSpec {
        channels: CHANNELS,
        sample_rate: SAMPLE_RATE,
        bits_per_sample: BITS_PER_SAMPLE,
        sample_format: hound::SampleFormat::Int,
    };
    let mut writer = hound::WavWriter::create(path, spec)?;
    for &sample in samples {
        writer.write_sample(sample)?;
    }
    writer.finalize()?;
    Ok(())
}
