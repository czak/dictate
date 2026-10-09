# dictate

Transcribes a WAV file with a local GGUF model supported by `transcribe-cpp`.

## Usage

```sh
cargo run -- transcribe --model <model.gguf> <audio.wav>
```

Example:

```sh
cargo run -- transcribe --model ~/.local/share/models/parakeet-unified-en-0.6b-Q8_0.gguf sample.wav
```

Run `dictate --help` or `dictate transcribe --help` for all options.

## Recording a sample with PipeWire

```sh
pw-record --channels 1 --rate 16000 --format s16 sample.wav
```

Press Ctrl+C to stop. The file must be mono, 16 kHz, 16-bit PCM.
