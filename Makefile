.PHONY: check fmt clippy build test

check: fmt clippy build test
	@echo "All checks passed."

fmt:
	cargo fmt -- --check

clippy:
	cargo clippy --all-targets -- -D warnings

build:
	cargo build --all-targets

test:
	cargo test
