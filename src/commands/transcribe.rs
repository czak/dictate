use transcribe_cpp::{Model, RunOptions};

use crate::audio::load_wav;
use crate::cli::TranscribeArgs;

pub fn run(args: &TranscribeArgs) -> Result<String, String> {
    let pcm = load_wav(&args.audio)?;
    let model = Model::load(&args.model).map_err(|err| err.to_string())?;
    let mut session = model.session().map_err(|err| err.to_string())?;
    let result = session
        .run(&pcm, &RunOptions::default())
        .map_err(|err| err.to_string())?;
    Ok(result.text.trim().to_string())
}
