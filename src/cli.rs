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
    /// Transcribe a WAV file and print the text to stdout.
    Transcribe(TranscribeArgs),
}

#[derive(Debug, clap::Args)]
pub struct TranscribeArgs {
    /// Path to the GGUF model file.
    #[arg(short, long, value_name = "PATH", value_parser = existing_file)]
    pub model: PathBuf,

    /// Mono, 16 kHz, 16-bit WAV file to transcribe.
    #[arg(value_name = "AUDIO", value_parser = existing_file)]
    pub audio: PathBuf,
}

fn existing_file(raw: &str) -> Result<PathBuf, String> {
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
