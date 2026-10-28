mod actions;
mod capture;
mod cli;
mod commands;
mod ctl;
mod indicator;
mod keyboard;
mod log;
mod server;
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
        Command::Serve { model } => commands::serve::run(model),
        Command::Record => ctl::record(),
        Command::Transcribe => ctl::transcribe(),
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            log::err!("{err:#}");
            ExitCode::FAILURE
        }
    }
}
