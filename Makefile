.PHONY: build test buildwin testwin

build:
	cargo fmt
	cargo build
	cp target/debug/llmaker .


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
