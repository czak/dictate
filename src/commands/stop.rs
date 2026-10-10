use anyhow::Result;

use crate::socket::{self, Request};

/// Asks the daemon to stop recording and prints the transcript.
pub fn run() -> Result<()> {
    println!("{}", socket::text(&Request::Stop)?);
    Ok(())
}
