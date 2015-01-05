.PHONY: build all test testextension debug

all: test

test:
	cargo test

testextension: testmodule.so
	python -c "import testmodule; print(repr(testmodule.__author__))"

debug: testmodule.so
	gdb --args python -c "import testmodule; print(repr(testmodule.__author__))"

target/librust-cpython-21cf8ea55e61f78d.rlib: src/*.rs Cargo.toml
	cargo build

testmodule.so: testmodule.rs target/librust-cpython-21cf8ea55e61f78d.rlib Makefile
	rustc testmodule.rs -g --extern rust-cpython=target/librust-cpython-21cf8ea55e61f78d.rlib --extern abort_on_panic=target/deps/libabort_on_panic-95c987fec9e5b445.rlib -o testmodule.so


