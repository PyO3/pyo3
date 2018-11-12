# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](http://keepachangelog.com/en/1.0.0/)
and this project adheres to [Semantic Versioning](http://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

 * Added a `wrap_module!` macro similar to the existing `wrap_function!` macro. Only available on python 3

### Changed
 * Renamed `add_function` to `add_wrapped` as it now also supports modules.

### Removed

 * `PyToken` was removed due to unsoundness (See [#94](https://github.com/PyO3/pyo3/issues/94)).
 * Removed the unnecessary type parameter from `PyObjectAlloc`

## [0.5.0] - 2018-11-11

### Added

 * `#[pyclass]` objects can now be returned from rust functions
 * `PyComplex` by kngwyu in [#226](https://github.com/PyO3/pyo3/pull/226)
 * `PyDict::from_sequence()`, equivalent to `dict([(key, val), ...])`
 * Bindings for the `datetime` standard library types: `PyDate`, `PyTime`, `PyDateTime`, `PyTzInfo`, `PyDelta` with associated `ffi` types, by pganssle [#200](https://github.com/PyO3/pyo3/pull/200).
 * `PyString`, `PyUnicode`, and `PyBytes` now have an `as_bytes()` method that returns `&[u8]`.
 * `PyObjectProtocol::get_type_ptr()` by ijl in [#242](https://github.com/PyO3/pyo3/pull/242)

### Removed
 * Removed most entries from the prelude. The new prelude is small and clear.
 * Slowly removing specialization uses
 * `PyString`, `PyUnicode`, and `PyBytes` no longer have a `data()` method
 (replaced by `as_bytes()`) and `PyStringData` has been removed.
 * The pyobject_extract macro

### Changed
 * Removes the types from the root module and the prelude. They now live in `pyo3::types` instead.
 * All exceptions are consturcted with `py_err` instead of `new`, as they return `PyErr` and not `Self`.
 * `as_mut` and friends take and `&mut self` instead of `&self`
 * `ObjectProtocol::call` now takes an `Option<&PyDict>` for the kwargs instead of an `IntoPyDictPointer`.
 * `IntoPyDictPointer` was replace by `IntoPyDict` which doesn't convert `PyDict` itself anymore and returns a `PyDict` instead of `*mut PyObject`.
 * `PyTuple::new` now takes an `IntoIterator` instead of a slice
 * Updated to syn 0.15
 * Splitted `PyTypeObject` into `PyTypeObject` without the create method and `PyTypeCreate` with requires `PyObjectAlloc<Self> + PyTypeInfo + Sized`.
 * Ran `cargo edition --fix` which prefixed path with `crate::` for rust 2018
 * Renamed `async` to `pyasync` as async will be a keyword in the 2018 edition.
 * Starting to use `NonNull<*mut PyObject>` for Py and PyObject by ijl [#260](https://github.com/PyO3/pyo3/pull/260)

### Fixed

 * Added an explanation that the GIL can temporarily be released even while holding a GILGuard.
 * Lots of clippy errors
 * Fix segfault on calling an unknown method on a PyObject
 * Work around a [bug](https://github.com/rust-lang/rust/issues/55380) in the rust compiler by kngwyu [#252](https://github.com/PyO3/pyo3/pull/252)
 * Fixed a segfault with subclassing pyo3 create classes and using `__class__` by kngwyu [#263](https://github.com/PyO3/pyo3/pull/263)

## [0.4.1] - 2018-08-20

### Fixed

 * Fixed compilation on nightly since `use_extern_macros` was stabilized

### Changed

 * PyTryFrom's error is always to `PyDowncastError`

### Removed

 * The pyobject_downcast macro

## [0.4.0] - 2018-07-30

### Removed

 * Conversions from tuples to PyDict due to [rust-lang/rust#52050](https://github.com/rust-lang/rust/issues/52050)

### Changed

 * Merged both examples into one
 * Rustfmt all the things :heavy_check_mark:
 * Switched to [Keep a Changelog](http://keepachangelog.com/en/1.0.0/)

## [0.3.2] - 2018-07-22

### Changed

* Replaced `concat_idents` with mashup

## [0.3.1] - 2018-07-18

### Fixed

* Fixed scoping bug in pyobject_native_type that would break rust-numpy

## [0.3.0] - 2018-07-18

### Changed

* Upgraded to syn 0.14 which means much better error messages :tada:
* 128 bit integer support by [kngwyu](https://github.com/kngwyu) ([#137](https://github.com/PyO3/pyo3/pull/173))
* `proc_macro` has been stabilized on nightly ([rust-lang/rust#52081](https://github.com/rust-lang/rust/pull/52081)). This means that we can remove the `proc_macro` feature, but now we need the `use_extern_macros` from the 2018 edition instead.
* All proc macro are now prefixed with `py` and live in the prelude. This means you can use `#[pyclass]`, `#[pymethods]`, `#[pyproto]`, `#[pyfunction]` and `#[pymodinit]` directly, at least after a `use pyo3::prelude::*`. They were also moved into a module called `proc_macro`. You shouldn't use `#[pyo3::proc_macro::pyclass]` or other longer paths in attributes because `proc_macro_path_invoc` isn't going to be stabilized soon.
* Renamed the `base` option in the `pyclass` macro to `extends`.
* `#[pymodinit]` uses the function name as module name, unless the name is overrriden with `#[pymodinit(name)]`
* The guide is now properly versioned.

### Added

* A few internal macros became part of the public api ([#155](https://github.com/PyO3/pyo3/pull/155), [#186](https://github.com/PyO3/pyo3/pull/186))
* Always clone in getters. This allows using the get-annotation on all Clone-Types

## [0.2.7] - 2018-05-18

### Fixed

* Fix nightly breakage with proc_macro_path

## [0.2.6] - 2018-04-03

### Fixed

* Fix compatibility with TryFrom trait #137

## [0.2.5] - 2018-02-21

### Added

* CPython 3.7 support

### Fixed

* Embedded CPython 3.7b1 crashes on initialization #110
* Generated extension functions are weakly typed #108
* call_method*() crashes when the method does not exist #113
* Allow importing exceptions from nested modules #116

## [0.2.4] - 2018-01-19

### Added

* Allow to get mutable ref from PyObject #106
* Drop `RefFromPyObject` trait
* Add Python::register_any() method

### Fixed

* Fix impl `FromPyObject` for `Py<T>`
* Mark method that work with raw pointers as unsafe #95

## [0.2.3] - 11-27-2017

### Fixed

* Proper `c_char` usage #93

### Changed

* Rustup to 1.23.0-nightly 2017-11-07

### Removed

* Remove use of now unneeded 'AsciiExt' trait


## [0.2.2] - 09-26-2017

### Changed

* Rustup to 1.22.0-nightly 2017-09-30

## [0.2.1] - 09-26-2017

### Fixed

* Fix rustc const_fn nightly breakage

## [0.2.0] - 08-12-2017

### Changed

* Allow to add gc support without implementing PyGCProtocol #57
* Refactor `PyErr` implementation. Drop `py` parameter from constructor.

### Added

* Added inheritance support #15
* Added weakref support #56
* Added subclass support #64
* Added `self.__dict__` supoort #68
* Added `pyo3::prelude` module #70
* Better `Iterator` support for PyTuple, PyList, PyDict #75
* Introduce IntoPyDictPointer similar to IntoPyTuple #69

## 0.1.0 - 07-23-2017

### Added

* Initial release

[Unreleased]: https://github.com/pyo3/pyo3/compare/v0.5.0...HEAD
[0.5.0]: https://github.com/pyo3/pyo3/compare/v0.4.1...v0.5.0
[0.4.1]: https://github.com/pyo3/pyo3/compare/v0.4.0...v0.4.1
[0.4.0]: https://github.com/pyo3/pyo3/compare/v0.3.2...v0.4.0
[0.3.2]: https://github.com/pyo3/pyo3/compare/v0.3.1...v0.3.2
[0.3.1]: https://github.com/pyo3/pyo3/compare/v0.3.0...v0.3.1
[0.3.0]: https://github.com/pyo3/pyo3/compare/v0.2.7...v0.3.0
[0.2.7]: https://github.com/pyo3/pyo3/compare/v0.2.6...v0.2.7
[0.2.6]: https://github.com/pyo3/pyo3/compare/v0.2.5...v0.2.6
[0.2.5]: https://github.com/pyo3/pyo3/compare/v0.2.4...v0.2.5
[0.2.4]: https://github.com/pyo3/pyo3/compare/v0.2.3...v0.2.4
[0.2.3]: https://github.com/pyo3/pyo3/compare/v0.2.2...v0.2.3
[0.2.2]: https://github.com/pyo3/pyo3/compare/v0.2.1...v0.2.2
[0.2.1]: https://github.com/pyo3/pyo3/compare/v0.2.0...v0.2.1
[0.2.0]: https://github.com/pyo3/pyo3/compare/v0.1.0...v0.2.0
