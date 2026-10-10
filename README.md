# dictate

Records from the default PipeWire input, transcribes it with a local GGUF
model supported by `transcribe-cpp`, and types the result into the focused
window through the Wayland virtual keyboard protocol.

## Requirements

- A Wayland compositor that offers `zwp_virtual_keyboard_manager_v1`
  (Sway and other wlroots compositors do).
- Only the first `wl_seat` is used. Multi-seat setups are not supported.

## Usage

Start the daemon once; it connects to Wayland, loads the model, and serves
requests on a Unix socket at `$XDG_RUNTIME_DIR/dictate.sock`:

```sh
cargo run -- daemon --model ~/.local/share/models/parakeet-ultra-Q8_0.gguf
```

Stop the daemon with Ctrl+C. Interrupting it while recording discards the
recording.

Run `dictate --help` or `dictate <command> --help` for all options.

## Dictating

With the daemon running, `start` begins recording from the default microphone
and returns immediately. `stop` ends the recording, transcribes it, and types
the text into the focused window:

```sh
dictate start
# ...speak...
dictate stop
```

`stop` returns only after the text has been typed. If typing fails, the
transcript is included in the error on stderr.

`start` fails if a recording is already in progress, and `stop` fails if none
is. Audio is held in the daemon's memory until `stop`.

`start` does not launch the daemon yet; run `dictate daemon` first.

### Sway binding

```
bindsym --release XF86Launch5 exec dictate stop
```

### How typing works

The daemon creates one virtual keyboard when it starts. Its keymap assigns one
key to each character in a fixed set: printable ASCII, Polish letters, and
common punctuation. Most text needs no keymap change. Other characters trigger
a keymap upload for that utterance, followed by a 10 ms pause so the focused
client can apply it. Without that pause, the client drops the first key.

Characters that cannot be typed, such as newlines, are skipped. Characters
with no key are also skipped, and the daemon reports how many.

## Checks

```sh
scripts/check.sh
```

Runs `rustfmt`, `clippy`, the build, and the unit tests. Transcription is not
covered by the checks; verify it by hand with `dictate start` and `dictate stop`.

The typing path needs a compositor. Its test is ignored by default:

```sh
DICTATE_TEST_TEXT="Zażółć gęślą" cargo test -- --include-ignored types_into_focused_window
```

Run it only against a private headless compositor, never against your desktop
session, because it types into whatever window is focused.
