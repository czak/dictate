use std::env;
use std::path::PathBuf;

/// Protocol: the client writes one line holding an absolute WAV path. The
/// daemon replies with `OK\n<text>` or `ERR\n<message>`, then closes.
pub const OK: &str = "OK";
pub const ERR: &str = "ERR";

/// $XDG_RUNTIME_DIR/dictate.sock, falling back to /tmp/dictate.sock.
pub fn default_path() -> PathBuf {
    match env::var_os("XDG_RUNTIME_DIR") {
        Some(dir) if !dir.is_empty() => PathBuf::from(dir).join("dictate.sock"),
        _ => PathBuf::from("/tmp/dictate.sock"),
    }
}
