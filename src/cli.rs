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

    /// Ask a running daemon to transcribe a WAV file and print the text.
    Transcribe {
        /// Mono, 16 kHz, 16-bit WAV file to transcribe.
        #[arg(value_parser = existing_file)]
        audio: PathBuf,

        /// Source language code (e.g. `en`, `pl`). Omit to let the model autodetect.
        /// Ignored by models that take no language.
        #[arg(short, long)]
        language: Option<String>,
    },

    /// Record from the default microphone via PipeWire until Ctrl+C.
    Record {
        /// Where to write the recording. Overwritten if it exists.
        output: PathBuf,
    },
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
