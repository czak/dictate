use std::path::Path;
use std::time::Instant;

use anyhow::Result;
use transcribe_cpp::{Model, disable_logging};

use crate::actions;
use crate::indicator::RecordingIndicator;
use crate::keyboard::Typist;
use crate::log;
use crate::server;
use crate::socket::Request;
use crate::wayland::Wayland;

/// The XKB layout the virtual keyboard types with. Fixed for now.
const LAYOUT: &str = "pl";

/// Connects the virtual keyboard, loads the model, then serves requests until
/// interrupted. Interrupting while recording discards the recording.
pub fn run(model_path: &Path) -> Result<()> {
    // The native library and ggml log to stderr by default. Silence them so
    // the output is only our own events. Must happen before the model loads.
    disable_logging();

    let wayland = Wayland::connect()?;
    let mut keyboard = Typist::new(&wayland, LAYOUT)?;
    let mut indicator = match RecordingIndicator::new(&wayland) {
        Ok(indicator) => Some(indicator),
        Err(err) => {
            log::warning!("No recording indicator: {err:#}");
            None
        }
    };
    let started = Instant::now();
    let mut session = Model::load(model_path)?.session()?;
    log::info!(
        "Loaded {} in {:.2}s",
        model_path.display(),
        started.elapsed().as_secs_f64()
    );
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
