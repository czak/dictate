mod audio;
mod cli;
mod commands;
mod socket;

use std::process::ExitCode;

use clap::Parser;

use cli::{Cli, Command};

fn main() -> ExitCode {
    let cli = Cli::parse();
    let result = match &cli.command {
        Command::Daemon(args) => commands::daemon::run(&args.model, &cli.socket),
        Command::Transcribe(args) => commands::transcribe::run(&cli.socket, &args.audio),
        Command::Record(args) => commands::record::run(&args.output),
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("{err:#}");
            ExitCode::FAILURE
        }
    }
}
