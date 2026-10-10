//! Wire protocol between `dictate transcribe` (client) and `dictate daemon` (server).
//!
//! Each message is one line of JSON. A client sends a [`Request`] and the daemon
//! answers with a [`Reply`], then closes the connection.

use std::env;
use std::fs;
use std::io::{BufRead, BufReader, Read, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::PathBuf;

use anyhow::{Context, Result, bail};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

/// A transcription job for the daemon.
#[derive(Debug, Serialize, Deserialize)]
pub struct Request {
    pub wav_path: PathBuf,
}

/// The daemon's answer to a [`Request`].
#[derive(Debug, Serialize, Deserialize)]
pub enum Reply {
    Text(String),
    Error(String),
}

/// `$XDG_RUNTIME_DIR/dictate.sock`.
pub fn path() -> Result<PathBuf> {
    match env::var_os("XDG_RUNTIME_DIR") {
        Some(dir) if !dir.is_empty() => Ok(PathBuf::from(dir).join("dictate.sock")),
        _ => bail!("XDG_RUNTIME_DIR must be set"),
    }
}

/// Sends `request` to the running daemon and returns the transcript.
pub fn transcribe(request: &Request) -> Result<String> {
    let socket = path()?;
    let stream = UnixStream::connect(&socket).with_context(|| {
        format!(
            "cannot reach daemon at {}; start it with `dictate daemon --model <model.gguf>`",
            socket.display()
        )
    })?;
    write_message(&stream, request)?;
    let reply: Reply =
        read_message(&stream)?.context("daemon closed the connection without replying")?;
    match reply {
        Reply::Text(text) => Ok(text),
        Reply::Error(message) => bail!("{message}"),
    }
}

/// Binds the socket, refusing if a live daemon owns it and replacing a stale socket file.
pub fn bind() -> Result<UnixListener> {
    let path = path()?;
    if path.exists() {
        if UnixStream::connect(&path).is_ok() {
            bail!("{}: daemon already running", path.display());
        }
        fs::remove_file(&path).with_context(|| format!("cannot remove {}", path.display()))?;
    }
    UnixListener::bind(&path).with_context(|| format!("cannot bind {}", path.display()))
}

/// Answers each incoming request with `handle`. Failed connections are logged
/// and do not stop the loop.
pub fn serve(listener: &UnixListener, mut handle: impl FnMut(Request) -> Reply) {
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                if let Err(err) = answer(&stream, &mut handle) {
                    eprintln!("request failed: {err:#}");
                }
            }
            Err(err) => eprintln!("accept failed: {err}"),
        }
    }
}

fn answer(stream: &UnixStream, handle: &mut impl FnMut(Request) -> Reply) -> Result<()> {
    // A connection with no request is a liveness probe from `bind`; nothing to answer.
    let Some(request) = read_message(stream)? else {
        return Ok(());
    };
    write_message(stream, &handle(request))
}

fn write_message(mut writer: impl Write, message: &impl Serialize) -> Result<()> {
    let mut line = serde_json::to_vec(message)?;
    line.push(b'\n');
    writer.write_all(&line)?;
    Ok(())
}

/// Reads one message. `None` means the peer sent an empty line or closed the connection.
fn read_message<T: DeserializeOwned>(reader: impl Read) -> Result<Option<T>> {
    let mut line = String::new();
    BufReader::new(reader).read_line(&mut line)?;
    if line.trim().is_empty() {
        return Ok(None);
    }
    Ok(Some(serde_json::from_str(&line)?))
}
