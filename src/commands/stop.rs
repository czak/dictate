use anyhow::Result;

use crate::socket::{self, Request};

/// Asks the daemon to stop recording. The daemon types the transcript.
pub fn run() -> Result<()> {
    socket::ack(&Request::Stop)
}
