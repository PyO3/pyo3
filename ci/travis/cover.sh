#!/bin/sh

### Setup kcov #################################################################

curl -SsL https://github.com/SimonKagstrom/kcov/archive/master.tar.gz | tar xz
cd kcov-master
cmake .
make
install src/kcov $HOME/.cargo/bin/kcov
cd $TRAVIS_BUILD_DIR
rm -rf kcov-master


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
