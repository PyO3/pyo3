#!/bin/bash

set -e -u -o pipefail

cargo test --features "${FEATURES:-} num-bigint num-complex"
(cd pyo3-derive-backend; cargo test)

for example_dir in examples/*; do
    cd $example_dir
    tox -c "tox.ini" -e py
    cd -
done
