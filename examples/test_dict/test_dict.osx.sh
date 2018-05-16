#!/bin/bash
DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
pushd ${DIR}
PYTHONPATH="${DIR}:${PYTHONPATH}"
RUST_BACKTRACE=1
RUSTC_FLAGS="-C link-arg=-undefined -C link-arg=dynamic_lookup"
cargo build && cp target/debug/libtest_dict.dylib _test_dict.so && python3 test_dict.py
popd
