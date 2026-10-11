use std::fs;
use std::path::Path;
use std::process;

use anyhow::{Context, Result, anyhow};
use transcribe_cpp::{Model, RunOptions, Session};

use crate::capture::Capture;
use crate::indicator::RecordingIndicator;
use crate::keyboard::Typist;
use crate::recorder::Recorder;
use crate::socket::{self, Reply, Request};
use crate::wayland::Wayland;

/// Connects the virtual keyboard, loads the model, then serves requests until
/// interrupted. Shows a recording indicator if the compositor supports it.
///
/// Interrupting while recording discards the recording.
pub fn run(model_path: &Path) -> Result<()> {
    let wayland = Wayland::connect()?;
    let mut keyboard = Typist::new(&wayland)?;
    let mut indicator = match RecordingIndicator::new(&wayland) {
        Ok(indicator) => Some(indicator),
        Err(err) => {
            eprintln!("no recording indicator: {err:#}");
            None
        }
    };
    let listener = socket::bind()?;
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
    socket::serve(&listener, |request| {
        let daemon = Daemon {
            session: &mut session,
            recorder: &mut recorder,
            keyboard: &mut keyboard,
            indicator: indicator.as_mut(),
        };
        match handle(daemon, request) {
            Ok(reply) => reply,
            Err(err) => Reply::Error(format!("{err:#}")),
        }
    });
    Ok(())
}

/// The daemon state that a request may touch.
struct Daemon<'a> {
    session: &'a mut Session,
    recorder: &'a mut Recorder<Capture>,
    keyboard: &'a mut Typist,
    indicator: Option<&'a mut RecordingIndicator>,
}

fn handle(daemon: Daemon, request: Request) -> Result<Reply> {
    let Daemon {
        session,
        recorder,
        keyboard,
        indicator,
    } = daemon;
    match request {
        Request::Start => {
            recorder.start(|| {
                Capture::start().context("cannot record from PipeWire; is it running?")
            })?;
            if let Some(indicator) = indicator
                && let Err(err) = indicator.show()
            {
                eprintln!("cannot show recording indicator: {err:#}");
            }
            Ok(Reply::Ok)
        }
        Request::Stop => {
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
