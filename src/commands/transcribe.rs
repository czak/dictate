use std::fs;
use std::io::{Read, Write};
use std::os::unix::net::UnixStream;
use std::path::Path;

use anyhow::{Context, Result, bail};

use crate::socket::{ERR, OK};

pub fn run(socket: &Path, audio: &Path, language: Option<&str>) -> Result<()> {
    let audio = fs::canonicalize(audio).with_context(|| audio.display().to_string())?;
    let language = language.unwrap_or("");
    if language.contains(['\t', '\n', '\r']) {
        bail!("invalid language code: {language:?}");
    }

    let mut stream = UnixStream::connect(socket).with_context(|| {
        format!(
            "cannot reach daemon at {}; start it with `dictate daemon --model <model.gguf>`",
            socket.display()
        )
    })?;
    writeln!(stream, "{language}\t{}", audio.display())?;

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
