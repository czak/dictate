use std::path::PathBuf;

pub const USAGE: &str = "\
Local speech-to-text using a GGUF model supported by transcribe.cpp.

Usage:
  dictate serve <MODEL>         Load the model and serve requests on the socket
  dictate ctl record            Start recording from the default microphone
  dictate ctl transcribe        Stop recording, transcribe, and type the text";

#[derive(Debug, PartialEq, Eq)]
pub enum Command {
    /// Load the model and serve requests on the socket.
    Serve { model: PathBuf },

    /// Send an action to a running server.
    Ctl(Action),
}

#[derive(Debug, PartialEq, Eq)]
pub enum Action {
    /// Start recording from the default microphone.
    Record,
    /// Stop recording, transcribe it, and type the text.
    Transcribe,
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
        "serve" => {
            let model = args.next().ok_or("serve: missing <MODEL>")?;
            Command::Serve {
                model: existing_file(model)?,
            }
        }
        "ctl" => Command::Ctl(ctl_action(args.next())?),
        _ => return Err(format!("unknown command: {name}")),
    };

    if let Some(extra) = args.next() {
        return Err(format!("unexpected argument: {extra}"));
    }

    Ok(command)
}

fn ctl_action(action: Option<String>) -> Result<Action, String> {
    match action.as_deref() {
        Some("record") => Ok(Action::Record),
        Some("transcribe") => Ok(Action::Transcribe),
        _ => Err("ctl: expected record or transcribe".into()),
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

    use super::{Action, Command, parse};

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
    fn ctl_record_and_transcribe() {
        assert_eq!(run(&["ctl", "record"]), Ok(Command::Ctl(Action::Record)));
        assert_eq!(
            run(&["ctl", "transcribe"]),
            Ok(Command::Ctl(Action::Transcribe))
        );
    }

    #[test]
    fn serve_takes_existing_model_file() {
        let model = model_path();
        assert_eq!(
            run(&["serve", &model]),
            Ok(Command::Serve {
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
    fn ctl_without_known_action_is_an_error() {
        assert_eq!(
            run(&["ctl"]),
            Err("ctl: expected record or transcribe".into())
        );
        assert_eq!(
            run(&["ctl", "start"]),
            Err("ctl: expected record or transcribe".into())
        );
    }

    #[test]
    fn serve_without_model_is_an_error() {
        assert_eq!(run(&["serve"]), Err("serve: missing <MODEL>".into()));
    }

    #[test]
    fn serve_model_must_exist() {
        assert_eq!(
            run(&["serve", "/nonexistent/model.gguf"]),
            Err("/nonexistent/model.gguf: not a file".into())
        );
    }

    #[test]
    fn extra_arguments_are_rejected() {
        let model = model_path();
        assert_eq!(
            run(&["ctl", "record", "now"]),
            Err("unexpected argument: now".into())
        );
        assert_eq!(
            run(&["serve", &model, "extra"]),
            Err("unexpected argument: extra".into())
        );
        assert_eq!(
            run(&["ctl", "transcribe", "-h"]),
            Err("unexpected argument: -h".into())
        );
    }
}
