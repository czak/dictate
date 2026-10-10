use std::fs;
use std::path::Path;
use std::process;

use anyhow::{Context, Result, anyhow};
use transcribe_cpp::{Model, RunOptions, Session};

use crate::capture::Capture;
use crate::keyboard::VirtualKeyboard;
use crate::recorder::Recorder;
use crate::socket::{self, Reply, Request};

/// Connects the virtual keyboard, loads the model, then serves requests until
/// interrupted.
///
/// Interrupting while recording discards the recording.
pub fn run(model_path: &Path) -> Result<()> {
    let mut keyboard = VirtualKeyboard::connect()?;
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
        match handle(&mut session, &mut recorder, &mut keyboard, request) {
            Ok(reply) => reply,
            Err(err) => Reply::Error(format!("{err:#}")),
        }
    });
    Ok(())
}

fn handle(
    session: &mut Session,
    recorder: &mut Recorder<Capture>,
    keyboard: &mut VirtualKeyboard,
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
