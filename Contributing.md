# Contributing

Thank you for your interest in contributing to PyO3! All are welcome - please consider reading our [Code of Conduct](https://github.com/PyO3/pyo3/blob/main/Code-of-Conduct.md) to keep our community positive and inclusive.

If you are searching for ideas how to contribute, proceed to the ["Getting started contributing"](#getting-started-contributing) section. If you have found a specific issue to contribute to and need information about the development process, you may find the section ["Writing pull requests"](#writing-pull-requests) helpful.

If you want to become familiar with the codebase, see
[Architecture.md](https://github.com/PyO3/pyo3/blob/main/Architecture.md).

## Getting started contributing

Please join in with any part of PyO3 which interests you. We use GitHub issues to record all bugs and ideas. Feel free to request an issue to be assigned to you if you want to work on it.

You can browse the API of the non-public parts of PyO3 [here](https://pyo3.netlify.app/internal/doc/pyo3/index.html).

The following sections also contain specific ideas on where to start contributing to PyO3.

## Setting up a development environment

To work and develop PyO3, you need Python & Rust installed on your system.
* We encourage the use of [rustup](https://rustup.rs/) to be able to select and choose specific toolchains based on the project.
* [Pyenv](https://github.com/pyenv/pyenv) is also highly recommended for being able to choose a specific Python version.
* [virtualenv](https://virtualenv.pypa.io/en/latest/) can also be used with or without Pyenv to use specific installed Python versions.
* [`nox`][nox] is used to automate many of our CI tasks.

### Help users identify bugs

The [PyO3 Discord server](https://discord.gg/33kcChzH7f) is very active with users who are new to PyO3, and often completely new to Rust. Helping them debug is a great way to get experience with the PyO3 codebase.

Helping others often reveals bugs, documentation weaknesses, and missing APIs. It's a good idea to open GitHub issues for these immediately so the resolution can be designed and implemented!

### Implement issues ready for development

Issues where the solution is clear and work is not in progress use the [needs-implementer](https://github.com/PyO3/pyo3/issues?q=is%3Aissue+is%3Aopen+label%3Aneeds-implementer) label.

Don't be afraid if the solution is not clear to you! The core PyO3 contributors will be happy to mentor you through any questions you have to help you write the solution.

### Help write great docs

PyO3 has a user guide (using mdbook) as well as the usual Rust API docs. The aim is for both of these to be detailed, easy to understand, and up-to-date. Pull requests are always welcome to fix typos, change wording, add examples, etc.

There are some specific areas of focus where help is currently needed for the documentation:

- Issues requesting documentation improvements are tracked with the [documentation](https://github.com/PyO3/pyo3/issues?q=is%3Aissue+is%3Aopen+label%3Adocumentation) label.
- Not all APIs had docs or examples when they were made. The goal is to have documentation on all PyO3 APIs ([#306](https://github.com/PyO3/pyo3/issues/306)). If you see an API lacking a doc, please write one and open a PR!

To build the docs (including all features), install [`nox`][nox] and then run

```shell
nox -s docs -- open
```

#### Doctests

We use lots of code blocks in our docs. Run `cargo test --doc` when making changes to check that
the doctests still work, or `cargo test` to run all the Rust tests including doctests. See
https://doc.rust-lang.org/rustdoc/documentation-tests.html for a guide on doctests.

#### Building the guide

You can preview the user guide by building it locally with `mdbook`.

First, install [`mdbook`][mdbook], the [`mdbook-tabs`][mdbook-tabs] plugin and [`nox`][nox]. Then, run

```shell
nox -s build-guide -- --open
```

To check all links in the guide are valid, also install [`lychee`][lychee] and use the `check-guide` session instead:

```shell
nox -s check-guide
```

### Help design the next PyO3

Issues which don't yet have a clear solution use the [needs-design](https://github.com/PyO3/pyo3/issues?q=is%3Aissue+is%3Aopen+label%3Aneeds-design) label.

If any of these issues interest you, please join in with the conversation on the issue! All opinions are valued, and if you're interested in going further with e.g. draft PRs to experiment with API designs, even better!

### Review pull requests

Everybody is welcome to submit comments on open PRs. Please help ensure new PyO3 APIs are safe, performant, tidy, and easy to use!

## Writing pull requests

Here are a few things to note when you are writing PRs.

### Testing and Continuous Integration

The PyO3 repo uses GitHub Actions. PRs are blocked from merging if CI is not successful. Formatting, linting and tests are checked for all Rust and Python code. In addition, all warnings in Rust code are disallowed (using `RUSTFLAGS="-D warnings"`).

Tests run with all supported Python versions with the latest stable Rust compiler, as well as for Python 3.9 with the minimum supported Rust version.

If you are adding a new feature, you should add it to the `full` feature in our *Cargo.toml** so that it is tested in CI.

You can run these checks yourself with `nox`. Use  `nox -l` to list the full set of subcommands you can run.

#### Linting Python code
`nox -s ruff`

#### Linting Rust code
`nox -s rustfmt`

#### Semver checks
`cargo semver-checks check-release`

#### Clippy
`nox -s clippy-all`

#### Tests
`nox -s test` or `cargo test` for Rust tests only, `nox -f pytests/noxfile.py -s test` for Python tests only

#### Check all conditional compilation
`nox -s check-feature-powerset`

#### UI Tests

PyO3 uses [`trybuild`](https://github.com/dtolnay/trybuild) to develop UI tests to capture error messages from the Rust compiler for some of the macro functionality.

Because there are several feature combinations for these UI tests, when updating them all (e.g. for a new Rust compiler version) it may be helpful to use the `update-ui-tests` nox session:

```bash
nox -s update-ui-tests
```

### Documenting changes

We use [towncrier](https://towncrier.readthedocs.io/en/stable/index.html) to generate a CHANGELOG for each release.

To include your changes in the release notes, you should create one (or more) news items in the `newsfragments` directory. Valid news items should be saved as `<PR>.<CATEGORY>.md` where `<PR>` is the pull request number and `<CATEGORY>` is one of the following:
- `packaging` - for dependency changes and Python / Rust version compatibility changes
- `added` - for new features
- `changed` - for features which already existed but have been altered or deprecated
- `removed` - for features which have been removed
- `fixed` - for "changed" features which were classed as a bugfix

Docs-only PRs do not need news items; start your PR title with `docs:` to skip the check.

### Style guide

#### Generic code

PyO3 has a lot of generic APIs to increase usability. These can come at the cost of generic code bloat. Where reasonable, try to implement a concrete sub-portion of generic functions. There are two forms of this:

- If the concrete sub-portion doesn't benefit from re-use by other functions, name it `inner` and keep it as a local to the function.
- If the concrete sub-portion is re-used by other functions, preferably name it `_foo` and place it directly below `foo` in the source code (where `foo` is the original generic function).

#### FFI calls

PyO3 makes a lot of FFI calls to Python's C API using raw pointers. Where possible try to avoid using pointers-to-temporaries in expressions:

```rust
// dangerous
pyo3::ffi::Something(name.to_object(py).as_ptr());

// because the following refactoring is a use-after-free error:
let name = name.to_object(py).as_ptr();
pyo3::ffi::Something(name)
```

Instead, prefer to bind the safe owned `PyObject` wrapper before passing to ffi functions:

```rust
let name: PyObject = name.to_object(py);
pyo3::ffi::Something(name.as_ptr())
// name will automatically be freed when it falls out of scope
```

## Python and Rust version support policy

PyO3 aims to keep sufficient compatibility to make packaging Python extensions built with PyO3 feasible on most common package managers.

To keep package maintainers' lives simpler, PyO3 will commit, wherever possible, to only adjust minimum supported Rust and Python versions at the same time. This bump will only come in an `0.x` release, roughly once per year, after the oldest supported Python version reaches its end-of-life. (Check https://endoflife.date/python for a clear timetable on these.)

Below are guidelines on what compatibility all PRs are expected to deliver for each language.

### Python

PyO3 supports all officially supported Python versions, as well as the latest PyPy3 release. All of these versions are tested in CI.

#### Adding support for new CPython versions

If you plan to add support for a pre-release version of CPython, here's a (non-exhaustive) checklist:

 - [ ] Wait until the last alpha release (usually alpha7), since ABI is not guranteed until the first beta release
 - [ ] Add prelease_ver-dev (e.g. `3.14-dev`) to `‎.github/workflows/ci.yml`, and bump version in `noxfile.py`, `pyo3-ffi/Cargo.toml` under `max-version` within  `[package.metadata.cpython]`, and `max` within `pyo3-ffi/build.rs`
- [ ] Add a new abi3-prerelease feature for the version (e.g. `abi3-py314`)
   - In `pyo3-build-config/Cargo.toml`, set abi3-most_current_stable to ["abi3-prerelease"] and abi3-prerelease to ["abi3"]
   - In `pyo3-ffi/Cargo.toml`, set abi3-most_current_stable to ["abi3-prerelease", "pyo3-build-config/abi3-most_current_stable"] and abi3-prerelease to ["abi3", "pyo3-build-config/abi3-prerelease"]
   - In `Cargo.toml`, set abi3-most_current_stable to ["abi3-prerelease", "pyo3-ffi/abi3-most_current_stable"] and abi3-prerelease to ["abi3", "pyo3-ffi/abi3-prerelease"]
 - [ ] Use `#[cfg(Py_prerelease])` (e.g. `#[cfg(Py_3_14)]`) and `#[cfg(not(Py_prerelease]))` to indicate changes between the stable branches of CPython and the pre-release
 - [ ] Do not add a Rust binding to any function, struct, or global variable prefixed with `_` in CPython's headers
 - [ ] Ping @ngoldbaum and @davidhewitt for assistance

### Rust

PyO3 aims to make use of up-to-date Rust language features to keep the implementation as efficient as possible.

The minimum Rust version supported will be decided when the release which bumps Python and Rust versions is made. At the time, the minimum Rust version will be set no higher than the lowest Rust version shipped in the current Debian, RHEL and Alpine Linux distributions.

CI tests both the most recent stable Rust version and the minimum supported Rust version. Because of Rust's stability guarantees this is sufficient to confirm support for all Rust versions in between.

## Benchmarking

PyO3 has two sets of benchmarks for evaluating some aspects of its performance. The benchmark suite is currently very small - please open PRs with new benchmarks if you're interested in helping to expand it!

First, there are Rust-based benchmarks located in the `pyo3-benches` subdirectory. You can run these benchmarks with:

    nox -s bench

Second, there is a Python-based benchmark contained in the `pytests` subdirectory. You can read more about it [here](https://github.com/PyO3/pyo3/tree/main/pytests).

## Code coverage

You can view what code is and isn't covered by PyO3's tests. We aim to have 100% coverage - please check coverage and add tests if you notice a lack of coverage!

- First, ensure the llvm-cov cargo plugin is installed. You may need to run the plugin through cargo once before using it with `nox`.
```shell
cargo install cargo-llvm-cov
cargo llvm-cov
```
- Then, generate an `lcov.info` file with
```shell
nox -s coverage -- lcov
```
You can install an IDE plugin to view the coverage. For example, if you use VSCode:
- Add the [coverage-gutters](https://marketplace.visualstudio.com/items?itemName=ryanluker.vscode-coverage-gutters) plugin.
- Add these settings to VSCode's `settings.json`:
```json
{
    "coverage-gutters.coverageFileNames": [
        "lcov.info",
        "cov.xml",
        "coverage.xml",
    ],
    "coverage-gutters.showLineCoverage": true
}
```
- You should now be able to see green highlights for code that is tested, and red highlights for code that is not tested.

## Sponsor this project

At the moment there is no official organisation that accepts sponsorship on PyO3's behalf. If you're seeking to provide significant funding to the PyO3 ecosystem, please reach out to us on [GitHub](https://github.com/PyO3/pyo3/issues/new) or [Discord](https://discord.gg/33kcChzH7f) and we can discuss.

In the meanwhile, some of our maintainers have personal GitHub sponsorship pages and would be grateful for your support:

- [davidhewitt](https://github.com/sponsors/davidhewitt)
- [messense](https://github.com/sponsors/messense)

[mdbook]: https://rust-lang.github.io/mdBook/cli/index.html
[mdbook-tabs]: https://mdbook-plugins.rustforweb.org/tabs.html
[lychee]: https://github.com/lycheeverse/lychee
[nox]: https://github.com/theacodes/nox
