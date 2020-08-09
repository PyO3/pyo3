#!/bin/bash
set -ex

# run `cargo test` only if testing against cpython.
if ! [[ $FEATURES == *"pypy"* ]]; then
  cargo test --features "$FEATURES num-bigint num-complex"
  ( cd pyo3-derive-backend; cargo test )
else
  # check that pypy at least builds
  cargo build;
fi

for example_dir in examples/*; do
    cd $example_dir
    if [[ $FEATURES == *"pypy"* ]]; then
        tox -c "tox.ini" -e pypy3
    else
        tox -c "tox.ini" -e py
    fi
    cd -
done
