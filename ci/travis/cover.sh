#!/bin/bash

set -ex

### PyPy does not run the test suite ###########################################

if [[ $FEATURES == *"pypy"* ]]; then
  exit 0
fi

### Run kcov ###################################################################

rm -f target/debug/pyo3*.d
rm -f target/debug/test_*.d
rm -f target/debug/test_doc-*

# Note: On travis this is run with -P1 because it started failing with
# `-P $(nproc)`. kcov can probably be run in parallel if used with different CI
# Exclude test_compile_error.rs (See https://github.com/PyO3/pyo3/pull/503)
FILES=$(find . -path ./target/debug/pyo3\* -or -path ./target/debug/test_\* -not -name '*test_compile_error*')
echo $FILES | xargs -n1 -P1 sh -c '
  dir="target/cov/$(basename $@)"
  mkdir -p $dir
  echo "Collecting coverage data of $(basename $@)"
  kcov \
    --exclude-path=./tests \
    --exclude-region="#[cfg(test)]:#[cfg(testkcovstopmarker)]" \
    --exclude-pattern=/.cargo,/usr/lib \
    --verify $dir "$@" 2>&1 >/dev/null
' _

### Upload coverage ############################################################

echo "Uploading code coverage"
curl -SsL https://codecov.io/bash | bash
