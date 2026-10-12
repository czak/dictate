use std::path::PathBuf;

pub const USAGE: &str = "\
Local speech-to-text using a GGUF model supported by transcribe.cpp.

Usage:
  dictate daemon <MODEL>   Load the model and serve requests on the socket
  dictate start            Start recording from the default microphone
  dictate stop             Stop recording, transcribe, and type the text";

#[derive(Debug, PartialEq, Eq)]
pub enum Command {
    /// Load the model and serve transcription requests on the socket.
    Daemon { model: PathBuf },

    /// Ask a running daemon to start recording from the default microphone.
    Start,

    /// Ask a running daemon to stop recording, transcribe it, and type the text
    /// into the focused window.
    Stop,
}

/// Parses the arguments after the program name. On error, returns a message
/// describing the problem; the caller prints it together with `USAGE`.
pub fn parse<I>(args: I) -> Result<Command, String>
where
    I: IntoIterator<Item = String>,
{
    let mut args = args.into_iter();
    let Some(name) = args.next() else {
        return Err("missing command".into());
    };

    let command = match name.as_str() {
        "daemon" => {
            let model = args.next().ok_or("daemon: missing <MODEL>")?;
            Command::Daemon {
                model: existing_file(model)?,
            }
        }
        "start" => Command::Start,
        "stop" => Command::Stop,
        _ => return Err(format!("unknown command: {name}")),
    };

    match args.next() {
        Some(extra) => Err(format!("unexpected argument: {extra}")),
        None => Ok(command),
    }
}

fn existing_file(raw: String) -> Result<PathBuf, String> {
    let path = PathBuf::from(raw);
    if path.is_file() {
        Ok(path)
    } else {
        Err(format!("{}: not a file", path.display()))
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::{Command, parse};

    fn run(args: &[&str]) -> Result<Command, String> {
        parse(args.iter().map(|arg| arg.to_string()))
    }

    fn model_path() -> String {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("Cargo.toml")
            .display()
            .to_string()
    }

    #[test]
    fn start_and_stop() {
        assert_eq!(run(&["start"]), Ok(Command::Start));
        assert_eq!(run(&["stop"]), Ok(Command::Stop));
    }

    #[test]
    fn daemon_takes_existing_model_file() {
        let model = model_path();
        assert_eq!(
            run(&["daemon", &model]),
            Ok(Command::Daemon {
                model: PathBuf::from(&model)
            })
        );
    }

    #[test]
    fn no_arguments_is_an_error() {
        assert_eq!(run(&[]), Err("missing command".into()));
    }

    #[test]
    fn unknown_command_is_an_error() {
        assert_eq!(run(&["record"]), Err("unknown command: record".into()));
        assert_eq!(run(&["--help"]), Err("unknown command: --help".into()));
    }

    #[test]
    fn daemon_without_model_is_an_error() {
        assert_eq!(run(&["daemon"]), Err("daemon: missing <MODEL>".into()));
    }

    #[test]
    fn daemon_model_must_exist() {
        assert_eq!(
            run(&["daemon", "/nonexistent/model.gguf"]),
            Err("/nonexistent/model.gguf: not a file".into())
        );
    }

    #[test]
    fn extra_arguments_are_rejected() {
        let model = model_path();
        assert_eq!(
            run(&["start", "now"]),
            Err("unexpected argument: now".into())
        );
        assert_eq!(
            run(&["daemon", &model, "extra"]),
            Err("unexpected argument: extra".into())
        );
        assert_eq!(run(&["stop", "-h"]), Err("unexpected argument: -h".into()));
    }
}
