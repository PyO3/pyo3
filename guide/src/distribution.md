# Distribution

## `setuptools-rust` integration

[`setuptools-rust`][setuptools-rust] is a setuptools helpers for Rust Python extensions. It supports `PyO3` by default.

For detailed usage, please refer to its [README](https://github.com/PyO3/setuptools-rust/blob/master/README.rst)

## Source distribution

To build a source code distribution, you need to add the following lines to your `MANIFEST.in` file to ensure it correctly packages Rust extension source code.

```text
include Cargo.toml
recursive-include src *
```

Then you can build a source code distribution by (assuming you have already written a `setup.py`):

```bash
python setup.py sdist
```

## Binary wheel distribution

To build a binary wheel, [manylinux](https://github.com/pypa/manylinux) would be a natural choice for Linux.

TODO: an manylinux1 example, macOS wheel

[setuptools-rust]: https://github.com/PyO3/setuptools-rust
