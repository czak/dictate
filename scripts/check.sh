#!/usr/bin/env bash
# Runs formatting, lint, build, and unit tests. Stops at the first failing step.
set -euo pipefail

repo=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
cd "$repo"

run() {
    local name=$1
    shift
    printf '\n==> %s\n' "$name"
    "$@"
    printf '    %s passed\n' "$name"
}

run "rustfmt" cargo fmt --check
run "clippy" cargo clippy --all-targets -- -D warnings
run "build" cargo build
run "unit tests" cargo test

printf '\nall checks passed\n'
