use anyhow::{Context, Result, anyhow};
use transcribe_cpp::{RunOptions, Session};

use crate::capture;
use crate::capture::Recording;
use crate::indicator::RecordingIndicator;
use crate::keyboard::Typist;

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
        eprintln!("cannot hide recording indicator: {err:#}");
    }
    let samples = capture::finish(recording);
    let text = run_model(session, &samples)?;
    // Keep the transcript in the error so it is not lost if typing fails.
    let skipped = keyboard
        .type_text(&text)
        .map_err(|err| anyhow!("cannot type transcript ({err:#}): {text}"))?;
    if skipped > 0 {
        eprintln!("skipped {skipped} characters that have no key");
    }
    Ok(())
}

fn run_model(session: &mut Session, samples: &[f32]) -> Result<String> {
    let result = session.run(samples, &RunOptions::default())?;
    Ok(result.text.trim().to_string())
}
