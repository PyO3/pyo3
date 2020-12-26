# Contributing

Thank you for your interest in contributing to PyO3! All are welcome - please consider reading our [Code of Conduct](Code-of-Conduct.md) to keep our community positive and inclusive.

If you are searching for ideas how to contribute, please read the "Getting started contributing" section. Once you've found an issue to contribute to, you may find the section "Writing pull requests" helpful.

## Getting started contributing

Please join in with any part of PyO3 which interests you. We use Github issues to record all bugs and ideas. Feel free to request an issue to be assigned to you if you want to work on it.

The following sections also contain specific ideas on where to start contributing to PyO3.

### Help users identify bugs

The [PyO3 Gitter channel](https://gitter.im/PyO3/Lobby) is very active with users who are new to PyO3, and often completely new to Rust. Helping them debug is a great way to get experience with the PyO3 codebase.

Helping others often reveals bugs, documentation weaknesses, and missing APIs. It's a good idea to open Github issues for these immediately so the resolution can be designed and implemented!

### Implement issues ready for development

Issues where the solution is clear and work is not in progress use the [needs-implementer](https://github.com/PyO3/pyo3/issues?q=is%3Aissue+is%3Aopen+label%3Aneeds-implemeter) label.

Don't be afraid if the solution is not clear to you! The core PyO3 contributors will be happy to mentor you through any questions you have to help you write the solution.

### Help write great docs

PyO3 has a user guide (using mdbook) as well as the usual Rust API docs. The aim is for both of these to be detailed, easy to understand, and up-to-date. Pull requests are always welcome to fix typos, change wording, add examples, etc.

There are some specific areas of focus where help is currently needed for the documentation:
- Issues requesting documentation improvements are tracked with the [documentation](https://github.com/PyO3/pyo3/issues?q=is%3Aissue+is%3Aopen+label%3Adocumentation) label.
- Not all APIs had docs or examples when they were made. The goal is to have documentation on all PyO3 APIs ([#306](https://github.com/PyO3/pyo3/issues/306)). If you see an API lacking a doc, please write one and open a PR!
- Not all `unsafe` APIs had safety notes when they made. We'd like to ensure all `unsafe` APIs are carefully explained ([#698](https://github.com/PyO3/pyo3/issues/698)). If you see an `unsafe` function missing safety notes, please write some and open a PR!

### Help design the next PyO3

Issues which don't yet have a clear solution use the [needs-design](https://github.com/PyO3/pyo3/issues?q=is%3Aissue+is%3Aopen+label%3Aneeds-design) label.

If any of these issues interest you, please join in with the conversation on the issue! All opinions are valued, and if you're interested in going further with e.g. draft PRs to experiment with API designs, even better!

### Review pull requests

Everybody is welcome to submit comments on open PRs. Please help ensure new PyO3 APIs are safe, performant, tidy, and easy to use!

## Writing pull requests

Here are a few things to note when you are writing PRs.

### Continuous Integration

The PyO3 repo uses Github Actions. PRs are blocked from merging if CI is not successful.

Formatting, linting and tests are checked for all Rust and Python code. In addition, all warnings in Rust code are disallowed (using `RUSTFLAGS="-D warnings"`).

Tests run with all supported Python versions with the latest stable Rust compiler, as well as for Python 3.9 with the minimum supported Rust version.

### Minimum supported Rust version

PyO3 aims to make use of up-to-date Rust language features to keep the implementation as efficient as possible.

However, there will always be support for at least the last few Rust compiler versions, so that users have time to update.

If your PR needs to bump the minimum supported Rust version, this is acceptable with the following conditions:
- Any changes which require a more recent version than what is [currently available on stable Red Hat Enterprise Linux](https://access.redhat.com/documentation/en-us/red_hat_developer_tools/1/) will be postponed. (This is to allow package managers to update support for newer `rustc` versions; RHEL was arbitrarily picked because their update policy is clear.)
- You might be asked to do extra work to tidy up other parts of the PyO3 codebase which can use the compiler version bump :)

## Benchmarking

PyO3 has two sets of benchmarks for evaluating some aspects of its performance. The benchmark suite is currently very small - please feel welcome to open PRs with new benchmarks if you're interested in helping to expand it!

First, there are Rust-based benchmarks located in the `benches` subdirectory. As long as you have a nightly rust compiler available on your system, you can run these benchmarks with:

    cargo +nightly bench

Second, there is a Python-based benchmark contained in the `word-count` example. You can read more about it [here](examples/word-count#benchmark).
