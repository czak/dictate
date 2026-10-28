.PHONY: check fmt clippy build test

check: fmt clippy build test

fmt:
	cargo fmt --check

clippy:
	cargo clippy --all-targets -- -D warnings

build:
	cargo build

test:
	cargo test
