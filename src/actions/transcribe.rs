use std::time::Instant;

use anyhow::{Context, Result, anyhow};
use transcribe_cpp::{RunOptions, Session};

use crate::capture;
use crate::capture::Recording;
use crate::indicator::RecordingIndicator;
use crate::keyboard::Typist;
use crate::log;

/// Ends the capture, transcribes it, and types the text. Fails if nothing is recording.
pub fn transcribe(
    recording: &mut Option<Recording>,
    session: &mut Session,
    keyboard: &mut Typist,
    indicator: Option<&mut RecordingIndicator>,
) -> Result<()> {
    let recording = recording.take().context("not recording")?;
    if let Some(indicator) = indicator
        && let Err(err) = indicator.hide()
    {
        log::warning!("Cannot hide recording indicator: {err:#}");
    }
    let samples = capture::finish(recording);
    log::info!(
        "Recording stopped. {:.2}s captured",
        capture::duration_secs(samples.len())
    );
    let text = run_model(session, &samples)?;
    // Keep the transcript in the error so it is not lost if typing fails.
    let skipped = keyboard
        .type_text(&text)
        .map_err(|err| anyhow!("cannot type transcript ({err:#}): {text}"))?;
    if skipped > 0 {
        log::warning!("Skipped {skipped} characters that have no key");
    }
    Ok(())
}

fn run_model(session: &mut Session, samples: &[f32]) -> Result<String> {
    let audio_secs = capture::duration_secs(samples.len());
    let started = Instant::now();
    let result = session.run(samples, &RunOptions::default())?;
    let elapsed_secs = started.elapsed().as_secs_f64();
    log::info!(
        "Transcribed in {elapsed_secs:.2}s ({:.0}x realtime)",
        audio_secs / elapsed_secs
    );
    Ok(result.text.trim().to_string())
}
