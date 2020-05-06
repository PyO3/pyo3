#!/bin/bash

set -ex

### PyPy does not run the test suite ###########################################

if [[ $FEATURES == *"pypy"* ]]; then
  exit 0
fi

### Run grcov ##################################################################
# export env vars and re-run tests
export CARGO_INCREMENTAL=0
export RUSTFLAGS="-Zpanic_abort_tests -Zprofile -Cpanic=abort -Ccodegen-units=1 -Cinline-threshold=0 -Clink-dead-code -Coverflow-checks=off"
export RUSTDOCFLAGS="-Cpanic=abort"
cargo test --features "$FEATURES num-bigint num-complex"

zip -0 ccov.zip `find . \( -name "pyo3*.gc*" \) -print`;
./grcov ccov.zip -s . -t lcov --llvm --branch --ignore-not-existing --ignore "/*" -o lcov.info;
bash <(curl -s https://codecov.io/bash) -f lcov.info;

