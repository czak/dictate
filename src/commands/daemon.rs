use std::fs;
use std::io::{self, BufRead, BufReader, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::{Path, PathBuf};
use std::process;

use anyhow::{Context, Result, bail};
use clap::Args as ClapArgs;
use transcribe_cpp::{Model, RunOptions, Session};

use crate::audio::load_wav;
use crate::cli::existing_file;
use crate::socket::{ERR, OK};

#[derive(Debug, ClapArgs)]
pub struct Args {
    /// Path to the GGUF model file.
    #[arg(short, long, env = "DICTATE_MODEL", value_name = "PATH", value_parser = existing_file)]
    pub model: PathBuf,
}

pub fn run(model_path: &Path, socket: &Path) -> Result<()> {
    let listener = bind(socket)?;

    let model = Model::load(model_path)?;
    let languages = model.capabilities().languages;
    if languages.is_empty() {
        eprintln!("model takes no language; language requests will be ignored");
    } else {
        eprintln!("supported languages: {}", languages.join(", "));
    }
    let mut session = model.session()?;

    let socket_path: PathBuf = socket.to_path_buf();
    ctrlc::set_handler(move || {
        let _ = fs::remove_file(&socket_path);
        process::exit(0);
    })?;

    eprintln!("listening on {}", socket.display());
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                if let Err(err) = handle(&mut session, &languages, stream) {
                    eprintln!("request failed: {err}");
                }
            }
            Err(err) => eprintln!("accept failed: {err}"),
        }
    }
    Ok(())
}

/// Binds the socket, refusing if a live daemon owns it and removing it if stale.
fn bind(socket: &Path) -> Result<UnixListener> {
    if socket.exists() {
        if UnixStream::connect(socket).is_ok() {
            bail!("{}: daemon already running", socket.display());
        }
        fs::remove_file(socket).with_context(|| socket.display().to_string())?;
    }
    UnixListener::bind(socket).with_context(|| socket.display().to_string())
}

fn handle(session: &mut Session, languages: &[String], stream: UnixStream) -> io::Result<()> {
    let mut request = String::new();
    BufReader::new(&stream).read_line(&mut request)?;
    let request = request.trim_end_matches(['\r', '\n']);
    if request.is_empty() {
        // Liveness probe from `bind` or a client that gave up; nothing to do.
        return Ok(());
    }
    let (language, path) = request.split_once('\t').unwrap_or(("", request));

    let reply = match transcribe(session, languages, language, Path::new(path)) {
        Ok(text) => format!("{OK}\n{text}"),
        Err(err) => format!("{ERR}\n{err:#}"),
    };
    (&stream).write_all(reply.as_bytes())
}

fn transcribe(
    session: &mut Session,
    languages: &[String],
    language: &str,
    audio: &Path,
) -> Result<String> {
    let language = resolve_language(languages, language)?;
    let pcm = load_wav(audio)?;
    let options = RunOptions {
        language,
        ..RunOptions::default()
    };
    let result = session.run(&pcm, &options)?;
    Ok(result.text.trim().to_string())
}

/// Maps the requested language onto what the model accepts. An empty request
/// means autodetect. Models without a language list ignore the request.
fn resolve_language(languages: &[String], requested: &str) -> Result<Option<String>> {
    if requested.is_empty() {
        return Ok(None);
    }
    if languages.is_empty() {
        eprintln!("warning: model takes no language; ignoring {requested:?}");
        return Ok(None);
    }
    if languages.iter().any(|l| l == requested) {
        return Ok(Some(requested.to_string()));
    }
    bail!(
        "language {requested:?} not supported by this model; supported: {}",
        languages.join(", ")
    )
}
