use anyhow::{Context, Result, bail};

use crate::capture::{self, Recording};
use crate::indicator::RecordingIndicator;
use crate::log;

/// Starts a capture and shows the recording indicator. Fails if one is already running.
pub fn record(
    recording: &mut Option<Recording>,
    indicator: Option<&mut RecordingIndicator>,
) -> Result<()> {
    if recording.is_some() {
        bail!("already recording");
    }

    let capture = capture::start().context("cannot record from the default input device")?;
    *recording = Some(capture);
    log::info!("Recording started");

    if let Some(indicator) = indicator
        && let Err(err) = indicator.show()
    {
        log::warning!("Cannot show recording indicator: {err:#}");
    }
    Ok(())
}
