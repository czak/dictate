use std::path::PathBuf;

use clap::{Parser, Subcommand};

use crate::commands::{daemon, transcribe};
use crate::socket;

/// Local speech-to-text using a GGUF model supported by transcribe.cpp.
#[derive(Debug, Parser)]
#[command(name = "dictate", version)]
pub struct Cli {
    /// Unix socket used by the daemon and the client.
    #[arg(
        short,
        long,
        global = true,
        env = "DICTATE_SOCKET",
        value_name = "PATH",
        default_value_os_t = socket::default_path()
    )]
    pub socket: PathBuf,

    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Load the model and serve transcription requests on the socket.
    Daemon(daemon::Args),
    /// Ask a running daemon to transcribe a WAV file and print the text.
    Transcribe(transcribe::Args),
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
