#!/bin/sh

set -ex

cargo build --features $FEATURES
cargo test --features $FEATURES

for example in examples/*; do
  cd $example
  python setup.py install
  pytest -v tests
  cd $TRAVIS_BUILD_DIR
done

cd tests/rustapi_module
tox -e py
cd $TRAVIS_BUILD_DIR
