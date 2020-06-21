.PHONY: build install demo test buildwin testwin

build:
	cargo fmt
	cargo build
	cp target/debug/llmaker .


clean:
	cargo clean
	@rm -rf llmaker

install:
	cargo install --path .

demo:
	llmaker demo/demo.mkr

test:
	cargo test
	./llmaker


buildwin:
	cargo fmt
	cargo build


testwin:
	cargo test
	target\debug\llmaker.exe -V
	target\debug\llmaker.exe -h
	target\debug\llmaker.exe test/test.mkr
