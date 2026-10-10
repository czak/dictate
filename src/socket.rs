use std::env;
use std::path::PathBuf;

/// Protocol: the client writes one line `<language>\t<absolute WAV path>`, where
/// the language is empty for autodetect. The daemon replies with
/// `OK\n<text>` or `ERR\n<message>`, then closes.
pub const OK: &str = "OK";
pub const ERR: &str = "ERR";

/// $XDG_RUNTIME_DIR/dictate.sock. Panics if XDG_RUNTIME_DIR is unset or empty.
pub fn path() -> PathBuf {
    let dir = env::var_os("XDG_RUNTIME_DIR")
        .filter(|dir| !dir.is_empty())
        .expect("XDG_RUNTIME_DIR must be set");
    PathBuf::from(dir).join("dictate.sock")
}
