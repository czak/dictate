use std::fs;
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::Path;
use std::process;

use anyhow::{Context, Result, anyhow, bail};
use transcribe_cpp::{Model, RunOptions, Session};

use crate::capture::Capture;
use crate::indicator::RecordingIndicator;
use crate::keyboard::Typist;
use crate::recorder::Recorder;
use crate::socket::{self, Reply, Request};
use crate::wayland::Wayland;

/// The XKB layout the virtual keyboard types with. Fixed for now.
const LAYOUT: &str = "pl";

/// Connects the virtual keyboard, loads the model, then serves requests until
/// interrupted. Shows a recording indicator if the compositor supports it.
///
/// Interrupting while recording discards the recording.
pub fn run(model_path: &Path) -> Result<()> {
    let wayland = Wayland::connect()?;
    let mut keyboard = Typist::new(&wayland, LAYOUT)?;
    let mut indicator = match RecordingIndicator::new(&wayland) {
        Ok(indicator) => Some(indicator),
        Err(err) => {
            eprintln!("no recording indicator: {err:#}");
            None
        }
    };
    let listener = bind()?;
    let mut session = Model::load(model_path)?.session()?;
    let mut recorder = Recorder::default();

    let socket_path = socket::path();
    ctrlc::set_handler({
        let socket_path = socket_path.clone();
        move || {
            let _ = fs::remove_file(&socket_path);
            process::exit(0);
        }
    })?;

    eprintln!("listening on {}", socket_path.display());
    accept_loop(&listener, |request| {
        let state = State {
            session: &mut session,
            recorder: &mut recorder,
            keyboard: &mut keyboard,
            indicator: indicator.as_mut(),
        };
        match handle(state, request) {
            Ok(reply) => reply,
            Err(err) => Reply::Error(format!("{err:#}")),
        }
    });
    Ok(())
}

/// The server state that a request may touch.
struct State<'a> {
    session: &'a mut Session,
    recorder: &'a mut Recorder<Capture>,
    keyboard: &'a mut Typist,
    indicator: Option<&'a mut RecordingIndicator>,
}

fn handle(state: State, request: Request) -> Result<Reply> {
    let State {
        session,
        recorder,
        keyboard,
        indicator,
    } = state;
    match request {
        Request::Record => {
            recorder.start(|| {
                Capture::start().context("cannot record from the default input device")
            })?;
            if let Some(indicator) = indicator
                && let Err(err) = indicator.show()
            {
                eprintln!("cannot show recording indicator: {err:#}");
            }
            Ok(Reply::Ok)
        }
        Request::Transcribe => {
            let recording = recorder.stop()?;
            if let Some(indicator) = indicator
                && let Err(err) = indicator.hide()
            {
                eprintln!("cannot hide recording indicator: {err:#}");
            }
            let samples = recording.finish();
            let text = transcribe(session, &samples)?;
            // Keep the transcript in the error so it is not lost if typing fails.
            let skipped = keyboard
                .type_text(&text)
                .map_err(|err| anyhow!("cannot type transcript ({err:#}): {text}"))?;
            if skipped > 0 {
                eprintln!("skipped {skipped} characters that have no key");
            }
            Ok(Reply::Ok)
        }
    }
}

fn transcribe(session: &mut Session, samples: &[f32]) -> Result<String> {
    let result = session.run(samples, &RunOptions::default())?;
    Ok(result.text.trim().to_string())
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

/// Answers each incoming request with `handle`. Failed connections are logged
/// and do not stop the loop.
fn accept_loop(listener: &UnixListener, mut handle: impl FnMut(Request) -> Reply) {
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
    let Some(request) = socket::read_message(stream)? else {
        return Ok(());
    };
    socket::write_message(stream, &handle(request))
}
