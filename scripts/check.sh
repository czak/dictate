#!/usr/bin/env bash
# Runs formatting, lint, build, unit tests, and an end-to-end transcription of
# /tmp/test.wav. Stops at the first failing step.
set -euo pipefail

repo=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
cd "$repo"

test_wav=/tmp/test.wav
# The only model permitted for testing (AGENTS.md).
model="$HOME/.local/share/models/parakeet-ultra-Q8_0.gguf"
expected="This is just some recording of my voice to verify that we're doing alright."

runtime=
daemon_pid=
cleanup() {
    if [[ -n $daemon_pid ]]; then
        kill "$daemon_pid" 2>/dev/null || true
        wait "$daemon_pid" 2>/dev/null || true
    fi
    if [[ -n $runtime ]]; then
        rm -rf "$runtime"
    fi
}
trap cleanup EXIT

run() {
    local name=$1
    shift
    printf '\n==> %s\n' "$name"
    "$@"
    printf '    %s passed\n' "$name"
}

round_trip() {
    if [[ ! -f $test_wav ]]; then
        echo "error: $test_wav is missing. Stop and ask the user to record it; do not substitute other audio." >&2
        exit 1
    fi

    # A private runtime dir keeps this daemon off the user's default socket.
    runtime=$(mktemp -d)
    XDG_RUNTIME_DIR=$runtime ./target/debug/dictate daemon --model "$model" \
        >"$runtime/daemon.log" 2>&1 &
    daemon_pid=$!

    local ready=
    for _ in $(seq 240); do
        if grep -q '^listening on' "$runtime/daemon.log"; then
            ready=1
            break
        fi
        if ! kill -0 "$daemon_pid" 2>/dev/null; then
            break
        fi
        sleep 0.5
    done
    if [[ -z $ready ]]; then
        cat "$runtime/daemon.log" >&2
        echo "error: daemon did not start listening" >&2
        exit 1
    fi

    XDG_RUNTIME_DIR=$runtime ./target/debug/dictate transcribe "$test_wav" >"$runtime/transcript.txt"
    diff -u <(printf '%s\n' "$expected") "$runtime/transcript.txt"
    echo "transcript matches expected text"
}

run "rustfmt" cargo fmt --check
run "clippy" cargo clippy --all-targets -- -D warnings
run "build" cargo build
run "unit tests" cargo test
run "round trip" round_trip

printf '\nall checks passed\n'
