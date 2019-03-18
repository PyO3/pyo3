#!/bin/bash
set -ex

cargo clean

# run `cargo test` only if testing against cpython.
if ! [[ $FEATURES == *"pypy"* ]]; then
  cargo test --features "$FEATURES num-complex"
  ( cd pyo3-derive-backend; cargo test )
fi

if [ $TRAVIS_JOB_NAME = 'Minimum nightly' ]; then
    cargo fmt --all -- --check
    cargo clippy --features "$FEATURES num-complex"
fi

if [[ $FEATURES == *"pypy"* ]]; then
    source activate pypy3
fi

for example_dir in examples/*; do
    tox -c "$example_dir/tox.ini" -e py
done
