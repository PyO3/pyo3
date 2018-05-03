#!/bin/sh

### Setup kcov #################################################################

curl -SsL https://github.com/SimonKagstrom/kcov/archive/master.tar.gz | tar xzv
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

for file in target/debug/pyo3-*[^\.d]; do _cover $file; done
for file in target/debug/test_*[^\.d]; do _cover $file; done


### Upload coverage ############################################################

echo "Uploading code coverage"
bash <(curl -s https://codecov.io/bash)
