#!/bin/sh

set -e

### Run kcov in parallel #######################################################

rm -f target/debug/pyo3-*.d
rm -f target/debug/test_*.d
rm -f target/debug/test_doc-*

# echo $FILES
FILES=$(find . -path ./target/debug/pyo3-\* -or -path ./target/debug/test_\*)
echo $FILES | xargs -n1 -P $(nproc) sh -c '
  dir="target/cov/$(basename $@)"
  mkdir -p $dir
  echo "Collecting coverage data of $(basename $@)"
  kcov --exclude-pattern=/.cargo,/usr/lib --verify $dir "$@" 2>&1 >/dev/null
' _

### Upload coverage ############################################################

echo "Uploading code coverage"
curl -SsL https://codecov.io/bash | bash
