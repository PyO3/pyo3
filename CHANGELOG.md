# Change Log

## [Unreleased]
- Added `pub` modifier to `py_class!` syntax: `py_class!(pub class ClassName |py| ...)`
- Changed `obj.extract::<Vec<T>>(py)` to work with any object implementing the sequence protocol; not just lists.
- Added the `buffer` module, which allows safe access to the [buffer protocol](https://docs.python.org/3/c-api/buffer.html).
  This allows zero-copy access to numpy arrays.
- When building with `--feature nightly`, `extract::<Vec<PrimitiveType>>` will try to use the buffer protocol
  before falling back to the sequence protocol.
- [Added support for optional parameters][81] to `py_argparse!`, `py_fn!` and `py_class!` macros. (PR by [@Luthaf])

  Example: `py_fn!(py, function(i: i32 = 0))`

[Unreleased]: https://github.com/dgrunwald/rust-cpython/compare/0.1.0...HEAD
[81]: https://github.com/dgrunwald/rust-cpython/pull/81
[@Luthaf]: https://github.com/Luthaf

## 0.1.0 - 2016-12-17
- First release that works on stable Rust.

