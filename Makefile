.PHONY: run

all:
	cargo build

run: all
	cargo run

test: all
	cargo test

release:
	cargo release
