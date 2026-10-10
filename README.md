# dictate

Transcribes a WAV file with a local GGUF model supported by `transcribe-cpp`.

## Usage

Start the daemon once; it loads the model and serves requests on a Unix socket
at `$XDG_RUNTIME_DIR/dictate.sock`:

```sh
cargo run -- daemon --model ~/.local/share/models/parakeet-ultra-Q8_0.gguf
```

Then transcribe through it. The text is printed to stdout:

```sh
cargo run -- transcribe sample.wav
```

Stop the daemon with Ctrl+C.

Run `dictate --help` or `dictate <command> --help` for all options.

## Recording a sample

`dictate record` captures from the default PipeWire input until Ctrl+C and writes a WAV file:

```sh
cargo run -- record sample.wav
```

Or use `pw-record`, which must write 32-bit float, since that is the only format `transcribe` accepts:

```sh
pw-record --channels 1 --rate 16000 --format f32 sample.wav
```

Press Ctrl+C to stop. The file must be mono, 16 kHz, 32-bit float.
