mod capture;
mod cli;
mod commands;
mod recorder;
mod socket;

use std::process::ExitCode;

use clap::Parser;

use cli::{Cli, Command};

fn main() -> ExitCode {
    let cli = Cli::parse();
    let result = match &cli.command {
        Command::Daemon { model } => commands::daemon::run(model),
        Command::Start => commands::start::run(),
        Command::Stop => commands::stop::run(),
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("{err:#}");
            ExitCode::FAILURE
        }
    }
}
