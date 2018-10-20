.PHONY: run

all:
	cargo build

run: all
	cargo run

test: all
	cargo test

release:
	@python3 make.py $(VERSION)
