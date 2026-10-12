mod capture;
mod cli;
mod commands;
mod indicator;
mod keyboard;
mod recorder;
mod socket;
mod wayland;

use std::process::ExitCode;

use cli::{Command, USAGE};

fn main() -> ExitCode {
    let command = match cli::parse(std::env::args().skip(1)) {
        Ok(command) => command,
        Err(message) => {
            eprintln!("dictate: {message}\n\n{USAGE}");
            return ExitCode::from(2);
        }
    };
    let result = match &command {
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
