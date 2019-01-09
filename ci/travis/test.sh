#!/bin/bash
# Note: Avoid using "-e" globally, its behaviour may sometimes be convoluted.
# For more details, see http://mywiki.wooledge.org/BashFAQ/105
set -x

cargo fmt --all -- --check || exit 1
cargo test --features "$FEATURES" || exit 1
cargo clippy --features "$FEATURES" || exit 1

for example_dir in examples/*; do
    tox -c "$example_dir/tox.ini" -e py || exit 1
done
