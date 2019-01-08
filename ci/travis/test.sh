#!/bin/bash

set -ex

cargo fmt --all -- --check
cargo test --features $FEATURES
cargo clippy --features $FEATURES

for example in examples/*; do
    tox -e py --workdir $example
done
