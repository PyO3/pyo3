# Contributing

Thank you for your interest in contributing to PyO3! All are welcome - please consider reading our [Code of Conduct](Code-of-Conduct.md) to keep our community positive and inclusive.

If you are searching for ideas how to contribute, proceed to the ["Getting started contributing"](#getting-started-contributing) section. If you have found a specific issue to contribute to and need information about the development process, you may find the section ["Writing pull requests"](#writing-pull-requests) helpful.

If you want to become familiar with the codebase, see
[Architecture.md](https://github.com/PyO3/pyo3/blob/main/Architecture.md).

## Getting started contributing

Please join in with any part of PyO3 which interests you. We use GitHub issues to record all bugs and ideas. Feel free to request an issue to be assigned to you if you want to work on it.

You can browse the API of the non-public parts of PyO3 [here](https://pyo3.rs/internal/doc/pyo3/index.html).

The following sections also contain specific ideas on where to start contributing to PyO3.

## Setting up a development environment

To work and develop PyO3, you need Python & Rust installed on your system.
* We encourage the use of [rustup](https://rustup.rs/) to be able to select and choose specific toolchains based on the project.
* [Pyenv](https://github.com/pyenv/pyenv) is also highly recommended for being able to choose a specific Python version.
* [virtualenv](https://virtualenv.pypa.io/en/latest/) can also be used with or without Pyenv to use specific installed Python versions.

### Caveats

* When using pyenv on macOS, installing a Python version using `--enable-shared` is required to make it work. i.e `env PYTHON_CONFIGURE_OPTS="--enable-shared" pyenv install 3.7.12`

### Help users identify bugs

The [PyO3 Gitter channel](https://gitter.im/PyO3/Lobby) is very active with users who are new to PyO3, and often completely new to Rust. Helping them debug is a great way to get experience with the PyO3 codebase.

Helping others often reveals bugs, documentation weaknesses, and missing APIs. It's a good idea to open GitHub issues for these immediately so the resolution can be designed and implemented!

### Implement issues ready for development

Issues where the solution is clear and work is not in progress use the [needs-implementer](https://github.com/PyO3/pyo3/issues?q=is%3Aissue+is%3Aopen+label%3Aneeds-implemeter) label.

Don't be afraid if the solution is not clear to you! The core PyO3 contributors will be happy to mentor you through any questions you have to help you write the solution.

### Help write great docs

PyO3 has a user guide (using mdbook) as well as the usual Rust API docs. The aim is for both of these to be detailed, easy to understand, and up-to-date. Pull requests are always welcome to fix typos, change wording, add examples, etc.

There are some specific areas of focus where help is currently needed for the documentation:

- Issues requesting documentation improvements are tracked with the [documentation](https://github.com/PyO3/pyo3/issues?q=is%3Aissue+is%3Aopen+label%3Adocumentation) label.
- Not all APIs had docs or examples when they were made. The goal is to have documentation on all PyO3 APIs ([#306](https://github.com/PyO3/pyo3/issues/306)). If you see an API lacking a doc, please write one and open a PR!

You can build the docs (including all features) with
```cargo xtask doc --open```

#### Doctests

We use lots of code blocks in our docs. Run `cargo test --doc` when making changes to check that
the doctests still work, or `cargo test` to run all the tests including doctests. See
https://doc.rust-lang.org/rustdoc/documentation-tests.html for a guide on doctests.

#### Building the guide

You can preview the user guide by building it locally with `mdbook`.

First, [install `mdbook`](https://rust-lang.github.io/mdBook/cli/index.html). Then, run
```mdbook build -d ../gh-pages-build guide --open```

### Help design the next PyO3

Issues which don't yet have a clear solution use the [needs-design](https://github.com/PyO3/pyo3/issues?q=is%3Aissue+is%3Aopen+label%3Aneeds-design) label.

If any of these issues interest you, please join in with the conversation on the issue! All opinions are valued, and if you're interested in going further with e.g. draft PRs to experiment with API designs, even better!

### Review pull requests

Everybody is welcome to submit comments on open PRs. Please help ensure new PyO3 APIs are safe, performant, tidy, and easy to use!

## Writing pull requests

Here are a few things to note when you are writing PRs.

### Continuous Integration

The PyO3 repo uses GitHub Actions. PRs are blocked from merging if CI is not successful.

Formatting, linting and tests are checked for all Rust and Python code. In addition, all warnings in Rust code are disallowed (using `RUSTFLAGS="-D warnings"`).

Tests run with all supported Python versions with the latest stable Rust compiler, as well as for Python 3.9 with the minimum supported Rust version.

If you are adding a new feature, you should add it to the `full` feature in our *Cargo.toml** so that it is tested in CI.

You can run these tests yourself with
```cargo xtask ci```
See [it's documentation](https://github.com/PyO3/pyo3/tree/main/xtask#readme)for more commands you can run.

## Python and Rust version support policy

PyO3 aims to keep sufficient compatibility to make packaging Python extensions built with PyO3 feasible on most common package managers.

To keep package maintainers' lives simpler, PyO3 will commit, wherever possible, to only adjust minimum supported Rust and Python versions at the same time. This bump will only come in an `0.x` release, roughly once per year, after the oldest supported Python version reaches its end-of-life. (Check https://endoflife.date/python for a clear timetable on these.)

Below are guidelines on what compatibility all PRs are expected to deliver for each language.

### Python

PyO3 supports all officially supported Python versions, as well as the latest PyPy3 release. All of these versions are tested in CI.

### Rust

PyO3 aims to make use of up-to-date Rust language features to keep the implementation as efficient as possible.

The minimum Rust version supported will be decided when the release which bumps Python and Rust versions is made. At the time, the minimum Rust version will be set no higher than the lowest Rust version shipped in the current Debian, RHEL and Alpine Linux distributions.

CI tests both the most recent stable Rust version and the minimum supported Rust version. Because of Rust's stability guarantees this is sufficient to confirm support for all Rust versions in between.

## Benchmarking

PyO3 has two sets of benchmarks for evaluating some aspects of its performance. The benchmark suite is currently very small - please open PRs with new benchmarks if you're interested in helping to expand it!

First, there are Rust-based benchmarks located in the `benches` subdirectory. As long as you have a nightly rust compiler available on your system, you can run these benchmarks with:

    cargo +nightly bench

Second, there is a Python-based benchmark contained in the `pytests` subdirectory. You can read more about it [here](pytests).

## Sponsor this project

At the moment there is no official organisation that accepts sponsorship on PyO3's behalf. If you're seeking to provide significant funding to the PyO3 ecosystem, please reach out to us on [GitHub](https://github.com/PyO3/pyo3/issues/new) or [Gitter](https://gitter.im/PyO3/Lobby) and we can discuss.

In the meanwhile, some of our maintainers have personal GitHub sponsorship pages and would be grateful for your support:

- [davidhewitt](https://github.com/sponsors/davidhewitt)
- [messense](https://github.com/sponsors/messense)
