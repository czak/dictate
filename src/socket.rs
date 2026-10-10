use std::env;
use std::path::PathBuf;

use anyhow::{Result, bail};

/// Protocol: the client writes one line `<language>\t<absolute WAV path>`, where
/// the language is empty for autodetect. The daemon replies with
/// `OK\n<text>` or `ERR\n<message>`, then closes.
pub const OK: &str = "OK";
pub const ERR: &str = "ERR";

/// $XDG_RUNTIME_DIR/dictate.sock.
pub fn path() -> Result<PathBuf> {
    match env::var_os("XDG_RUNTIME_DIR") {
        Some(dir) if !dir.is_empty() => Ok(PathBuf::from(dir).join("dictate.sock")),
        _ => bail!("XDG_RUNTIME_DIR is not set"),
    }
}
