//! Wire protocol shared by `dictate ctl` (client) and `dictate serve` (server).
//!
//! Each message is one line of JSON. A client sends a [`Request`] and the server
//! answers with a [`Reply`], then closes the connection.

use std::env;
use std::io::{BufRead, BufReader, Read, Write};
use std::os::unix::net::UnixStream;
use std::path::PathBuf;

use anyhow::{Context, Result};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

/// A job for the server.
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Request {
    /// Begin recording from the default microphone.
    Record,
    /// End the recording, transcribe it, and type the text into the focused window.
    Transcribe,
}

/// The server's answer to a [`Request`].
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Reply {
    /// The request succeeded and has no output.
    Ok,
    Error(String),
}

/// `$XDG_RUNTIME_DIR/dictate.sock`.
///
/// Panics if `XDG_RUNTIME_DIR` is unset.
pub fn path() -> PathBuf {
    let dir = env::var_os("XDG_RUNTIME_DIR").expect("XDG_RUNTIME_DIR must be set");
    PathBuf::from(dir).join("dictate.sock")
}

/// Connects to the server, sends `request`, and returns its reply.
pub fn send(request: &Request) -> Result<Reply> {
    let socket = path();
    let stream = UnixStream::connect(&socket).with_context(|| {
        format!(
            "cannot reach server at {}; start it with `dictate serve <model.gguf>`",
            socket.display()
        )
    })?;
    write_message(&stream, request)?;
    read_message(&stream)?.context("server closed the connection without replying")
}

pub fn write_message(mut writer: impl Write, message: &impl Serialize) -> Result<()> {
    let mut line = serde_json::to_vec(message)?;
    line.push(b'\n');
    writer.write_all(&line)?;
    Ok(())
}

/// Reads one message. `None` means the peer sent an empty line or closed the connection.
pub fn read_message<T: DeserializeOwned>(reader: impl Read) -> Result<Option<T>> {
    let mut line = String::new();
    BufReader::new(reader).read_line(&mut line)?;
    if line.trim().is_empty() {
        return Ok(None);
    }
    Ok(Some(serde_json::from_str(&line)?))
}
