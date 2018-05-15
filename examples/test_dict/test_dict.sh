#!/bin/bash
DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
pushd ${DIR}
PYTHONPATH="${DIR}:${PYTHONPATH}"
RUST_BACKTRACE=1
cargo build && cp target/debug/libtest_dict.so _test_dict.so && python3 test_dict.py
popd
