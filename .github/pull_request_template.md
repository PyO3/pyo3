Thank you for contributing to pyo3!

Please consider adding the following to your pull request:
 - an entry in CHANGELOG.md
 - docs to all new functions and / or detail in the guide
 - tests for all new or changed functions

Be aware our the CI pipeline will check your pull request for the following:
 - Rust tests (`cargo test`)
 - Rust lints (`cargo clippy --all -- -Dwarnings`)
 - Rust formatting (`cargo fmt`)
 - Python formatting (`black --check`. You can install black with `pip install black`)
 - Compatibility with all supported Python versions for all examples. This uses `tox`; you can do run it using `make test_py`.

You can run a similar set of checks as the CI pipeline using `make test`.
