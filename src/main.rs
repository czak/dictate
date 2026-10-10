mod cli;
mod commands;
mod socket;
mod wav;

use std::process::ExitCode;

use clap::Parser;

use cli::{Cli, Command};

fn main() -> ExitCode {
    let cli = Cli::parse();
    let result = socket::path().and_then(|socket| match &cli.command {
        Command::Daemon { model } => commands::daemon::run(model, &socket),
        Command::Transcribe { audio, language } => {
            commands::transcribe::run(&socket, audio, language.as_deref())
        }
        Command::Record { output } => commands::record::run(output),
    });
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("{err:#}");
            ExitCode::FAILURE
        }
    }
}
