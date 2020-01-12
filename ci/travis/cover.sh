#!/bin/bash

set -ex

### PyPy does not run the test suite ###########################################

if [[ $FEATURES == *"pypy"* ]]; then
  exit 0
fi

### Run grcov ##################################################################

zip -0 ccov.zip `find . \( -name "pyo3*.gc*" \) -print`;
./grcov ccov.zip -s . -t lcov --llvm --branch --ignore-not-existing --ignore "/*" -o lcov.info;
bash <(curl -s https://codecov.io/bash) -f lcov.info;

