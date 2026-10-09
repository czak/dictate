use std::path::Path;

/// Loads a mono, 16 kHz, 16-bit WAV file as normalized f32 samples.
pub fn load_wav(path: &Path) -> Result<Vec<f32>, String> {
    let mut reader =
        hound::WavReader::open(path).map_err(|err| format!("{}: {err}", path.display()))?;
    let spec = reader.spec();
    if (spec.channels, spec.sample_rate, spec.bits_per_sample) != (1, 16_000, 16) {
        return Err(format!(
            "{}: expected mono 16 kHz 16-bit, got {} channel(s) {} Hz {}-bit",
            path.display(),
            spec.channels,
            spec.sample_rate,
            spec.bits_per_sample,
        ));
    }
    reader
        .samples::<i16>()
        .map(|sample| sample.map(|sample| sample as f32 / 32768.0))
        .collect::<Result<Vec<_>, _>>()
        .map_err(|err| format!("{}: {err}", path.display()))
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::load_wav;

    fn write_wav(name: &str, channels: u16, sample_rate: u32, bits: u16) -> PathBuf {
        let path = std::env::temp_dir().join(format!("dictate-test-{name}.wav"));
        let spec = hound::WavSpec {
            channels,
            sample_rate,
            bits_per_sample: bits,
            sample_format: hound::SampleFormat::Int,
        };
        let mut writer = hound::WavWriter::create(&path, spec).unwrap();
        for _ in 0..16 {
            writer.write_sample(0i16).unwrap();
        }
        writer.finalize().unwrap();
        path
    }

    #[test]
    fn accepts_mono_16khz_16bit() {
        let path = write_wav("ok", 1, 16_000, 16);
        let samples = load_wav(&path).unwrap();
        assert_eq!(samples.len(), 16);
    }

    #[test]
    fn rejects_stereo() {
        let path = write_wav("stereo", 2, 16_000, 16);
        assert!(
            load_wav(&path)
                .unwrap_err()
                .contains("expected mono 16 kHz 16-bit")
        );
    }

    #[test]
    fn rejects_wrong_sample_rate() {
        let path = write_wav("rate", 1, 44_100, 16);
        assert!(
            load_wav(&path)
                .unwrap_err()
                .contains("expected mono 16 kHz 16-bit")
        );
    }

    #[test]
    fn rejects_8bit() {
        let path = write_wav("8bit", 1, 16_000, 8);
        assert!(
            load_wav(&path)
                .unwrap_err()
                .contains("expected mono 16 kHz 16-bit")
        );
    }
}
