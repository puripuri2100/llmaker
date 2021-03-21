.PHONY: build install demo test buildwin testwin

build:
	cargo fmt
	cargo build


clean:
	cargo clean
	@rm -rf llmaker

install:
	cargo install --path .

demo:
	llmaker demo/demo.mkr

test:
	cargo test
	cargo run -- test/test.mkr
	cargo run -- test/parser.mkr
