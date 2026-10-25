use anyhow::{Context, Result, bail};

use crate::capture::{self, Recording};
use crate::indicator::RecordingIndicator;

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

    if let Some(indicator) = indicator
        && let Err(err) = indicator.show()
    {
        eprintln!("cannot show recording indicator: {err:#}");
    }
    Ok(())
}
