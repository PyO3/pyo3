#!/bin/bash
set -ex

cargo fmt --all -- --check
cargo test --features "$FEATURES"
cargo clippy --features "$FEATURES"

for example_dir in examples/*; do
    tox -c "$example_dir/tox.ini" -e py
done
