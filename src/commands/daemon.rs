use std::fs;
use std::path::Path;
use std::process;

use anyhow::Result;
use transcribe_cpp::{Model, RunOptions, Session};

use crate::socket::{self, Reply, Request};
use crate::wav;

/// Loads the model, then serves transcription requests until interrupted.
pub fn run(model_path: &Path) -> Result<()> {
    let listener = socket::bind()?;
    let mut session = Model::load(model_path)?.session()?;

    let socket_path = socket::path()?;
    ctrlc::set_handler({
        let socket_path = socket_path.clone();
        move || {
            let _ = fs::remove_file(&socket_path);
            process::exit(0);
        }
    })?;

    eprintln!("listening on {}", socket_path.display());
    socket::serve(&listener, |request| {
        match transcribe(&mut session, &request) {
            Ok(text) => Reply::Text(text),
            Err(err) => Reply::Error(format!("{err:#}")),
        }
    });
    Ok(())
}

fn transcribe(session: &mut Session, request: &Request) -> Result<String> {
    let pcm = wav::load(&request.wav_path)?;
    let result = session.run(&pcm, &RunOptions::default())?;
    Ok(result.text.trim().to_string())
}
