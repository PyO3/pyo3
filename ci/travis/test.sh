#!/bin/sh


for example in examples/*; do
  cd $example
  python setup.py install
  pytest -v tests
  cd $TRAVIS_BUILD_DIR
done
