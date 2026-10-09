use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::sync::{Arc, Mutex};

use anyhow::{Context, Result};
use clap::Args as ClapArgs;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{BufferSize, HostId, StreamConfig};

use crate::audio::{CHANNELS, SAMPLE_RATE, write_wav};

#[derive(Debug, ClapArgs)]
pub struct Args {
    /// Where to write the recording. Overwritten if it exists.
    #[arg(value_name = "OUTPUT")]
    pub output: PathBuf,
}

pub fn run(output: &Path) -> Result<()> {
    let samples = capture().context("cannot record from PipeWire; is it running?")?;
    write_wav(output, &samples).with_context(|| format!("cannot write {}", output.display()))
}

/// Records mono 16 kHz S16 audio until Ctrl+C (SIGINT or SIGTERM) is received.
fn capture() -> Result<Vec<i16>> {
    let host = cpal::host_from_id(HostId::PipeWire)?;
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
    let stream = device.build_input_stream::<i16, _, _>(
        config,
        move |data: &[i16], _| sink.lock().unwrap().extend_from_slice(data),
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
