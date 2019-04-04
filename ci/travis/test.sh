#!/bin/bash
set -ex

cargo clean

# run `cargo test` only if testing against cpython.
if ! [[ $FEATURES == *"pypy"* ]]; then
  cargo test --features "$FEATURES num-complex"
  ( cd pyo3-derive-backend; cargo test )
else
  # check that pypy at least builds
  PYTHON_SYS_EXECUTABLE="/opt/anaconda/envs/pypy3/bin/pypy3" cargo build;
fi

if [ "$TRAVIS_JOB_NAME" = "Minimum nightly" ]; then
    cargo fmt --all -- --check
    cargo clippy --features "$FEATURES num-complex"
fi

for example_dir in examples/*; do
    tox -c "$example_dir/tox.ini" -e py
done
