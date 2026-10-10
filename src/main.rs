mod cli;
mod commands;
mod socket;
mod wav;

use std::process::ExitCode;

use clap::Parser;

use cli::{Cli, Command};

fn main() -> ExitCode {
    let cli = Cli::parse();
    let result = match &cli.command {
        Command::Daemon { model } => commands::daemon::run(model),
        Command::Transcribe { audio } => commands::transcribe::run(audio),
        Command::Record { output } => commands::record::run(output),
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("{err:#}");
            ExitCode::FAILURE
        }
    }
}
