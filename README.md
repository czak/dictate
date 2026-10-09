# dictate

Transcribes a WAV file with a local GGUF model supported by `transcribe-cpp`.

## Usage

Start the daemon once; it loads the model and serves requests on a Unix socket
(`$XDG_RUNTIME_DIR/dictate.sock`, or `/tmp/dictate.sock`; override with `--socket` or `DICTATE_SOCKET`):

```sh
cargo run -- daemon --model <model.gguf>
```

Then transcribe through it. The text is printed to stdout:

```sh
cargo run -- transcribe sample.wav
```

For multilingual models, pick the language per request with `-l`/`--language`
(e.g. `en`, `pl`). Omit it to autodetect. Models without a language list ignore it.

```sh
cargo run -- transcribe -l pl sample.wav
```

`--model` can also be set with `DICTATE_MODEL`. Stop the daemon with Ctrl+C.

Run `dictate --help` or `dictate <command> --help` for all options.

## Recording a sample with PipeWire

```sh
pw-record --channels 1 --rate 16000 --format s16 sample.wav
```

Press Ctrl+C to stop. The file must be mono, 16 kHz, 16-bit PCM.
