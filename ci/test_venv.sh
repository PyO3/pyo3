#!/bin/bash
set -ex

python -m venv venv
source venv/bin/activate
python -m pip install setuptools-rust

for example_dir in examples/*; do
    cd ${example_dir}
    python setup.py install
    cd -
done

rm -r venv
