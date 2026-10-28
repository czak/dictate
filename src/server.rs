use std::fs;
use std::os::unix::net::{UnixListener, UnixStream};
use std::process;

use anyhow::{Context, Result, bail};

use crate::log;
use crate::socket::{self, Reply, Request};

/// Binds the socket and answers each request with `dispatch`. A failing
/// dispatch is sent back as `Reply::Error`. Runs until interrupted.
pub fn serve(mut dispatch: impl FnMut(Request) -> Result<()>) -> Result<()> {
    let listener = bind()?;
    let socket_path = socket::path();
    ctrlc::set_handler({
        let socket_path = socket_path.clone();
        move || {
            let _ = fs::remove_file(&socket_path);
            process::exit(0);
        }
    })?;

    log::info!("Listening on {}", socket_path.display());
    // Failed connections are logged and do not stop the loop.
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                // A connection with no request is a liveness probe from `bind`; nothing to answer.
                let request = match socket::read_message(&stream) {
                    Ok(Some(request)) => request,
                    Ok(None) => continue,
                    Err(err) => {
                        log::warning!("Request failed: {err:#}");
                        continue;
                    }
                };
                let reply = match dispatch(request) {
                    Ok(()) => Reply::Ok,
                    Err(err) => Reply::Error(format!("{err:#}")),
                };
                if let Err(err) = socket::write_message(&stream, &reply) {
                    log::warning!("Request failed: {err:#}");
                }
            }
            Err(err) => log::err!("Accept failed: {err}"),
        }
    }
    Ok(())
}

/// Binds the socket, refusing if a live server owns it and replacing a stale socket file.
fn bind() -> Result<UnixListener> {
    let path = socket::path();
    if path.exists() {
        if UnixStream::connect(&path).is_ok() {
            bail!("{}: server already running", path.display());
        }
        fs::remove_file(&path).with_context(|| format!("cannot remove {}", path.display()))?;
    }
    UnixListener::bind(&path).with_context(|| format!("cannot bind {}", path.display()))
}
