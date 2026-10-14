mod capture;
mod cli;
mod ctl;
mod indicator;
mod keyboard;
mod keymap;
mod recorder;
mod server;
mod socket;
mod wayland;

use std::process::ExitCode;

use cli::{Action, Command, USAGE};

fn main() -> ExitCode {
    let command = match cli::parse(std::env::args().skip(1)) {
        Ok(command) => command,
        Err(message) => {
            eprintln!("dictate: {message}\n\n{USAGE}");
            return ExitCode::from(2);
        }
    };
    let result = match &command {
        Command::Serve { model } => server::run(model),
        Command::Ctl(Action::Record) => ctl::record(),
        Command::Ctl(Action::Transcribe) => ctl::transcribe(),
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("{err:#}");
            ExitCode::FAILURE
        }
    }
}
