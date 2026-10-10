use std::path::Path;
use std::sync::mpsc;
use std::sync::{Arc, Mutex};

use anyhow::{Context, Result};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{BufferSize, HostId, StreamConfig};

use crate::wav::{self, SPEC};

pub fn run(output: &Path) -> Result<()> {
    let samples = capture().context("cannot record from PipeWire; is it running?")?;
    wav::write(output, &samples).with_context(|| format!("cannot write {}", output.display()))
}

/// Records mono 16 kHz f32 audio until Ctrl+C (SIGINT or SIGTERM) is received.
fn capture() -> Result<Vec<f32>> {
    let host = cpal::host_from_id(HostId::PipeWire)?;
    let device = host
        .default_input_device()
        .context("no default input device")?;
    let config = StreamConfig {
        channels: SPEC.channels,
        sample_rate: SPEC.sample_rate,
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

    let (stop_tx, stop_rx) = mpsc::channel();
    ctrlc::set_handler(move || {
        let _ = stop_tx.send(());
    })?;

    stream.play()?;
    eprintln!("recording… press Ctrl+C to stop");
    let _ = stop_rx.recv();
    drop(stream);

    Ok(std::mem::take(&mut *samples.lock().unwrap()))
}
