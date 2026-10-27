# dictate

<img src="assets/icon.svg" width="96" align="right" alt="Icon">

Super simple dictation for Linux.

1. Talk to your microphone
2. Transcribe locally using [transcribe.cpp](https://github.com/handy-computer/transcribe.cpp)
3. Have text appear in active window

## Requirements

Runs on Wayland and expects the protocols:
- `zwp_virtual_keyboard_manager_v1` for the typing
- `zwlr_layer_shell_v1` for the recording indicator

Tested on Sway.

## Usage

The `serve` process loads the given model, does the recording and handles the Wayland connection.

```sh
dictate serve ~/.local/share/models/parakeet-unified-en-0.6b-Q8_0.gguf
```

Then use `record` and `transcribe` to start and stop recording, respectively.

Begin recording from the default input:

```sh
dictate record
```

Stop recording, transcribe, and immediately type the text into the active window:

```sh
dictate transcribe
```

You probably want to bind them to keys in your desktop environment, for example:

### Push-to-talk on Sway

Hold the key to record, release it to transcribe:

```
bindsym XF86AudioRecord exec dictate record
bindsym --release XF86AudioRecord exec dictate transcribe
```

### Push-to-talk on Hyprland

Hold the key to record, release it to transcribe:

```
bind = , XF86AudioRecord, exec, dictate record
bindr = , XF86AudioRecord, exec, dictate transcribe
```

## How to build

```sh
cargo build --release
```

## Current limitations

- I have good results with the `vulkan` backend, so that is currently enabled. If you don't like that, just revert to:

  ```diff
  -transcribe-cpp = { version = "0.3.1", features = ["vulkan"] }
  +transcribe-cpp = "0.3.1"
  ```
- The keyboard logic mimics actual keyboard layout and is currently hardcoded to `pl`. Chromium did not like made up keymaps.

## (Maybe) to do

- [ ] Language hint for multi-language models (something like `dictate transcribe <lang>`).
- [ ] Vocabulary (will depend on model support in transcribe.cpp)
