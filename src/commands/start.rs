use anyhow::Result;

use crate::socket::{self, Request};

/// Asks the daemon to start recording. Returns once recording has begun.
pub fn run() -> Result<()> {
    socket::ack(&Request::Start)
}
