#!/bin/sh

### Run kcov ###################################################################

_cover() {
    dir="target/cov/$(basename $@)"
    mkdir -p $dir
    kcov --exclude-pattern=/.cargo,/usr/lib --verify $dir "$@"
}

rm target/debug/pyo3-*.d
rm target/debug/test_*.d
rm target/debug/test_doc-*

for file in target/debug/pyo3-*; do _cover $file; done
for file in target/debug/test_*; do _cover $file; done


### Upload coverage ############################################################

echo "Uploading code coverage"
curl -SsL https://codecov.io/bash | bash
