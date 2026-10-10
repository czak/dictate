use std::fs;
use std::path::Path;
use std::process;

use anyhow::{Context, Result};
use transcribe_cpp::{Model, RunOptions, Session};

use crate::capture::Capture;
use crate::recorder::Recorder;
use crate::socket::{self, Reply, Request};

/// Loads the model, then serves requests until interrupted.
///
/// Interrupting while recording discards the recording.
pub fn run(model_path: &Path) -> Result<()> {
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
        match handle(&mut session, &mut recorder, request) {
            Ok(reply) => reply,
            Err(err) => Reply::Error(format!("{err:#}")),
        }
    });
    Ok(())
}

fn handle(
    session: &mut Session,
    recorder: &mut Recorder<Capture>,
    request: Request,
) -> Result<Reply> {
    match request {
        Request::Start => {
            recorder.start(|| {
                Capture::start().context("cannot record from PipeWire; is it running?")
            })?;
            Ok(Reply::Ok)
        }
        Request::Stop => {
            let samples = recorder.stop()?.finish();
            Ok(Reply::Text(transcribe(session, &samples)?))
        }
    }
}

fn transcribe(session: &mut Session, samples: &[f32]) -> Result<String> {
    let result = session.run(samples, &RunOptions::default())?;
    Ok(result.text.trim().to_string())
}
