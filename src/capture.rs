use std::sync::{Arc, Mutex};

use anyhow::{Context, Result};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{BufferSize, Stream, StreamConfig};

/// The only audio format the model accepts: mono, 16 kHz, f32 samples.
const CHANNELS: u16 = 1;
const SAMPLE_RATE: u32 = 16_000;

/// An in-progress recording from the default input device.
/// Dropping it without calling [`Capture::finish`] discards the audio.
pub struct Capture {
    stream: Stream,
    samples: Arc<Mutex<Vec<f32>>>,
}

impl Capture {
    /// Starts recording mono 16 kHz f32 audio.
    pub fn start() -> Result<Self> {
        let host = cpal::default_host();
        let device = host
            .default_input_device()
            .context("no default input device")?;
        let config = StreamConfig {
            channels: CHANNELS,
            sample_rate: SAMPLE_RATE,
            buffer_size: BufferSize::Default,
        };

        let samples = Arc::new(Mutex::new(Vec::new()));
        let sink = Arc::clone(&samples);
        let stream = device.build_input_stream::<f32, _, _>(
            config,
            move |data: &[f32], _| sink.lock().unwrap().extend_from_slice(data),
            |err| eprintln!("stream error: {err}"),
            None,
        )?;
        stream.play()?;
        Ok(Self { stream, samples })
    }

    /// Stops recording and returns everything captured.
    pub fn finish(self) -> Vec<f32> {
        drop(self.stream);
        std::mem::take(&mut *self.samples.lock().unwrap())
    }
}
