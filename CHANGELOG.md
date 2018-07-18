# Changelog

## 0.3.0 (unreleased)

* Upgraded to syn 0.14 which means much better error messages :tada:
* 128 bit integer support by [kngwyu](https://github.com/kngwyu) ([#137](https://github.com/PyO3/pyo3/pull/173))
* `proc_macro` has been stabilized on nightly ([rust-lang/rust#52081](https://github.com/rust-lang/rust/pull/52081)). This means that we can remove the `proc_macro` feature, but now we need the `use_extern_macros` from the 2018 edition instead.
* All proc macro are now prefixed with `py` and live in the prelude. This means you can use `#[pyclass]`, `#[pymethods]`, `#[pyproto]`, `#[pyfunction]` and `#[pymodinit]` directly, at least after a `use pyo3::prelude::*`. They were also moved into a module called `proc_macro`. You shouldn't use `#[pyo3::proc_macro::pyclass]` or other longer paths in attributes because `proc_macro_path_invoc` isn't going to be stabilized soon.
* Renamed the `base` option in the `pyclass` macro to `extends`.
* `#[pymodinit]` uses the function name as module name, unless the name is overrriden with `#[pymodinit(name)]`
* The guide is now properly versioned.
* A few internal macros became part of the public api ([#155](https://github.com/PyO3/pyo3/pull/155), [#186](https://github.com/PyO3/pyo3/pull/186))
* Always clone in getters. This allows using the get-annotation on all Clone-Types

## 0.2.7 (2018-05-18)

* Fix nightly breakage with proc_macro_path

## 0.2.6 (2018-04-03)

* Fix compatibility with TryFrom trait #137

## 0.2.5 (2018-02-21)

* CPython 3.7 support
* Embedded CPython 3.7b1 crashes on initialization #110
* Generated extension functions are weakly typed #108
* call_method*() crashes when the method does not exist #113
* Allow importing exceptions from nested modules #116

## 0.2.4 (2018-01-19)

* Allow to get mutable ref from PyObject #106
* Drop `RefFromPyObject` trait
* Add Python::register_any() method
* Fix impl `FromPyObject` for `Py<T>`
* Mark method that work with raw pointers as unsafe #95


## 0.2.3 (11-27-2017)

* Proper `c_char` usage #93
* Remove use of now unneeded 'AsciiExt' trait
* Rustup to 1.23.0-nightly 2017-11-07

## 0.2.2 (09-26-2017)

* Rustup to 1.22.0-nightly 2017-09-30

## 0.2.1 (09-26-2017)

* Fix rustc const_fn nightly breakage

## 0.2.0 (08-12-2017)

* Added inheritance support #15
* Added weakref support #56
* Allow to add gc support without implementing PyGCProtocol #57
* Refactor `PyErr` implementation. Drop `py` parameter from constructor.
* Added subclass support #64
* Added `self.__dict__` supoort #68
* Added `pyo3::prelude` module #70
* Better `Iterator` support for PyTuple, PyList, PyDict #75
* Introduce IntoPyDictPointer similar to IntoPyTuple #69

## 0.1.0 (07-23-2017)

* Initial release
