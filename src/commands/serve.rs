use std::path::Path;

use anyhow::Result;
use transcribe_cpp::Model;

use crate::actions;
use crate::indicator::RecordingIndicator;
use crate::keyboard::Typist;
use crate::server;
use crate::socket::Request;
use crate::wayland::Wayland;

/// The XKB layout the virtual keyboard types with. Fixed for now.
const LAYOUT: &str = "pl";

/// Connects the virtual keyboard, loads the model, then serves requests until
/// interrupted. Interrupting while recording discards the recording.
pub fn run(model_path: &Path) -> Result<()> {
    let wayland = Wayland::connect()?;
    let mut keyboard = Typist::new(&wayland, LAYOUT)?;
    let mut indicator = match RecordingIndicator::new(&wayland) {
        Ok(indicator) => Some(indicator),
        Err(err) => {
            eprintln!("no recording indicator: {err:#}");
            None
        }
    };
    let mut session = Model::load(model_path)?.session()?;
    let mut recording = None;

    server::serve(move |request| match request {
        Request::Record => actions::record(&mut recording, indicator.as_mut()),
        Request::Transcribe => actions::transcribe(
            &mut recording,
            &mut session,
            &mut keyboard,
            indicator.as_mut(),
        ),
    })
}
