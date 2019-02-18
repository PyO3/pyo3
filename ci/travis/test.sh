#!/bin/bash
set -ex

cargo test --features "$FEATURES num-complex"
if [ $TRAVIS_JOB_NAME = 'Minimum nightly' ]; then
    cargo fmt --all -- --check
    cargo clippy --features "$FEATURES num-complex"
fi

for example_dir in examples/*; do
    tox -c "$example_dir/tox.ini" -e py
done
