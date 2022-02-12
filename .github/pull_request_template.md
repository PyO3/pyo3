Thank you for contributing to pyo3!

Please consider adding the following to your pull request:
 - an entry in CHANGELOG.md
 - docs to all new functions and / or detail in the guide
 - tests for all new or changed functions

Be aware the CI pipeline will check your pull request for the following. This is done using `nox` (you can install with `pip install nox`):
 - Rust tests (`cargo test` or `nox -s test-rust`)
 - Examples (`nox -s test-py`)
 - Rust lints (`nox -s clippy`)
 - Rust formatting (`nox -s fmt-rust`)
 - Python formatting (`nox -s fmt-py`)
