# dictate

Records from the default PipeWire input and transcribes it with a local GGUF
model supported by `transcribe-cpp`.

## Usage

Start the daemon once; it loads the model and serves requests on a Unix socket
at `$XDG_RUNTIME_DIR/dictate.sock`:

```sh
cargo run -- daemon --model ~/.local/share/models/parakeet-ultra-Q8_0.gguf
```

Stop the daemon with Ctrl+C. Interrupting it while recording discards the
recording.

Run `dictate --help` or `dictate <command> --help` for all options.

## Dictating

With the daemon running, `start` begins recording from the default microphone
and returns immediately. `stop` ends the recording, transcribes it, and prints
the text to stdout:

```sh
dictate start
# ...speak...
dictate stop
```

`start` fails if a recording is already in progress, and `stop` fails if none
is. Audio is held in the daemon's memory until `stop`.

`start` does not launch the daemon yet; run `dictate daemon` first.

## Checks

```sh
scripts/check.sh
```

Runs `rustfmt`, `clippy`, the build, and the unit tests. Transcription is not
covered by the checks; verify it by hand with `dictate start` and `dictate stop`.
