#!/bin/sh

set -ex

cargo build --features $FEATURES
cargo test --features $FEATURES

for example in examples/*; do
  cd $example
  if [ -f tox.ini ]; then
      tox -e py
  else
    pip install -e .
    pytest -v tests
  fi
  cd $TRAVIS_BUILD_DIR
done
