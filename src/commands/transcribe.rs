use std::fs;
use std::path::Path;

use anyhow::{Context, Result};

use crate::socket::{self, Request};

pub fn run(audio: &Path) -> Result<()> {
    let request = Request {
        wav_path: fs::canonicalize(audio)
            .with_context(|| format!("cannot open {}", audio.display()))?,
    };
    println!("{}", socket::transcribe(&request)?);
    Ok(())
}
