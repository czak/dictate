use std::sync::{Arc, Mutex};

use anyhow::{Context, Result};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{BufferSize, Stream, StreamConfig};

use crate::log;

/// The only audio format the model accepts: mono, 16 kHz, f32 samples.
const CONFIG: StreamConfig = StreamConfig {
    channels: 1,
    sample_rate: 16_000,
    buffer_size: BufferSize::Default,
};

/// An in-progress recording. Dropping it without calling [`finish`] discards the audio.
pub type Recording = (Stream, Arc<Mutex<Vec<f32>>>);

/// Starts recording mono 16 kHz f32 audio from the default input device.
pub fn start() -> Result<Recording> {
    let host = cpal::default_host();
    let device = host
        .default_input_device()
        .context("no default input device")?;

    let samples = Arc::new(Mutex::new(Vec::new()));
    let sink = Arc::clone(&samples);
    let stream = device.build_input_stream::<f32, _, _>(
        CONFIG,
        move |data: &[f32], _| sink.lock().unwrap().extend_from_slice(data),
        |err| log::err!("Stream error: {err}"),
        None,
    )?;
    stream.play()?;
    Ok((stream, samples))
}

/// Length in seconds of `samples` captured samples.
pub fn duration_secs(samples: usize) -> f64 {
    samples as f64 / f64::from(CONFIG.sample_rate)
}

/// Stops recording and returns everything captured.
pub fn finish((stream, samples): Recording) -> Vec<f32> {
    drop(stream);
    std::mem::take(&mut *samples.lock().unwrap())
}
