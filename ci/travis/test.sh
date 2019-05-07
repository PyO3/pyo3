#!/bin/bash
set -ex

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
    cd $example_dir
    if [[ $FEATURES == *"pypy"* ]]; then
        tox -c "tox.ini" -e pypy3
    else
        tox -c "tox.ini" -e py
    fi
    cd -
done
