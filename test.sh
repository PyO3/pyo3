#!/bin/bash
set -xeu
cargo build
rustc testmodule.rs -L target/debug -L target/debug/deps -o testmodule.so
python -c "import testmodule; print(repr(testmodule.__author__))"

