.PHONY: run

all:
	cargo build

run: all
	cargo run
