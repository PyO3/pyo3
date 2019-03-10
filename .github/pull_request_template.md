Thank you for contributing to pyo3!

Here are some things you should check for submitting your pull request:

 - Run `cargo fmt` (This is checked by travis ci)
 - Run `cargo clippy` and check there are no hard errors (There are a bunch of existing warnings; This is also checked by travis)
 - If applicable, add an entry in the changelog.
 - If applicable, add documentation to all new items and extend the guide.
 - If applicable, add tests for all new or fixed functions

You might want to run `tox` (`pip install tox`) locally to check compatibility with all supported python versions. If you're using linux or mac you might find the Makefile helpful for testing.
