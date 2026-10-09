use std::env;
use std::path::Path;
use std::process::ExitCode;

use transcribe_cpp::{Model, RunOptions};

fn main() -> ExitCode {
    match run() {
        Ok(transcript) => {
            println!("{transcript}");
            ExitCode::SUCCESS
        }
        Err(err) => {
            eprintln!("{err}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<String, String> {
    let mut args = env::args().skip(1);
    let (Some(model_path), Some(audio_path)) = (args.next(), args.next()) else {
        return Err(usage());
    };
    if args.next().is_some() {
        return Err(usage());
    }

    require_file(&model_path)?;
    require_file(&audio_path)?;

    let pcm = load_wav(Path::new(&audio_path))?;
    let model = Model::load(&model_path).map_err(|err| err.to_string())?;
    let mut session = model.session().map_err(|err| err.to_string())?;
    let result = session
        .run(&pcm, &RunOptions::default())
        .map_err(|err| err.to_string())?;
    Ok(result.text.trim().to_string())
}

fn usage() -> String {
    "usage: dictate <model.gguf> <audio.wav>".to_string()
}

fn require_file(path: &str) -> Result<(), String> {
    if Path::new(path).is_file() {
        Ok(())
    } else {
        Err(format!("{path}: not a file"))
    }
}

fn load_wav(path: &Path) -> Result<Vec<f32>, String> {
    let mut reader =
        hound::WavReader::open(path).map_err(|err| format!("{}: {err}", path.display()))?;
    let spec = reader.spec();
    if (spec.channels, spec.sample_rate, spec.bits_per_sample) != (1, 16_000, 16) {
        return Err(format!(
            "{}: expected mono 16 kHz 16-bit, got {} channel(s) {} Hz {}-bit",
            path.display(),
            spec.channels,
            spec.sample_rate,
            spec.bits_per_sample,
        ));
    }
    reader
        .samples::<i16>()
        .map(|sample| sample.map(|sample| sample as f32 / 32768.0))
        .collect::<Result<Vec<_>, _>>()
        .map_err(|err| format!("{}: {err}", path.display()))
}
