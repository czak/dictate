//! Client for `dictate ctl`: sends one request to the server and reports the outcome.

use anyhow::{Result, bail};

use crate::socket::{self, Reply, Request};

/// Asks the server to start recording. Returns once recording has begun.
pub fn record() -> Result<()> {
    send(&Request::Record)
}

/// Asks the server to stop recording, transcribe it, and type the text.
pub fn transcribe() -> Result<()> {
    send(&Request::Transcribe)
}

fn send(request: &Request) -> Result<()> {
    match socket::send(request)? {
        Reply::Ok => Ok(()),
        Reply::Error(message) => bail!("{message}"),
    }
}
