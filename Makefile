.PHONY: build install demo test buildwin testwin

build:
	cargo fmt
	cargo build


clean:
	cargo clean

install:
	cargo install --path .

demo:
	llmaker demo/demo.mkr

test:
	cargo fmt
	cargo check
	cargo test
	cargo clippy
	cargo run -- test/test.mkr
	cargo run -- test/parser.mkr
