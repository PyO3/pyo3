#!/bin/bash
set -xeu
#@c:\cygwin\bin\touch src/lib.rs
cargo build
rustc testmodule.rs --extern rust-cpython=target/librust-cpython-21cf8ea55e61f78d.rlib -o testmodule.so
python -c "import testmodule; print(repr(testmodule.__author__))"

