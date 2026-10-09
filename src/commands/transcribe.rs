use std::fs;
use std::io::{Read, Write};
use std::os::unix::net::UnixStream;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use clap::Args as ClapArgs;

use crate::cli::existing_file;
use crate::socket::{ERR, OK};

#[derive(Debug, ClapArgs)]
pub struct Args {
    /// Mono, 16 kHz, 16-bit WAV file to transcribe.
    #[arg(value_name = "AUDIO", value_parser = existing_file)]
    pub audio: PathBuf,
}

pub fn run(socket: &Path, audio: &Path) -> Result<()> {
    let audio = fs::canonicalize(audio).with_context(|| audio.display().to_string())?;

    let mut stream = UnixStream::connect(socket).with_context(|| {
        format!(
            "cannot reach daemon at {}; start it with `dictate daemon --model <model.gguf>`",
            socket.display()
        )
    })?;
    writeln!(stream, "{}", audio.display())?;

    let mut reply = String::new();
    stream.read_to_string(&mut reply)?;

    match reply.split_once('\n') {
        Some((OK, text)) => {
            println!("{text}");
            Ok(())
        }
        Some((ERR, message)) => bail!("{message}"),
        _ => bail!("malformed reply from daemon: {reply:?}"),
    }
}
