use std::path::PathBuf;

use clap::{Parser, Subcommand};

/// Local speech-to-text using a GGUF model supported by transcribe.cpp.
#[derive(Debug, Parser)]
#[command(name = "dictate", version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Load the model and serve transcription requests on the socket.
    Daemon {
        /// Path to the GGUF model file.
        #[arg(short, long, value_parser = existing_file)]
        model: PathBuf,
    },

    /// Ask a running daemon to start recording from the default microphone.
    Start,

    /// Ask a running daemon to stop recording, transcribe it, and type the text
    /// into the focused window.
    Stop,
}

/// Value parser shared by the commands: accepts only paths to existing files.
pub fn existing_file(raw: &str) -> Result<PathBuf, String> {
    let path = PathBuf::from(raw);
    if path.is_file() {
        Ok(path)
    } else {
        Err(format!("{raw}: not a file"))
    }
}

#[cfg(test)]
mod tests {
    use clap::CommandFactory;

    use super::Cli;

    #[test]
    fn cli_definition_is_valid() {
        Cli::command().debug_assert();
    }
}
