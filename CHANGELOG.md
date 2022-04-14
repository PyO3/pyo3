# Changelog

All notable changes to this project will be documented in this file. For help with updating to new
PyO3 versions, please see the [migration guide](https://pyo3.rs/latest/migration.html).

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/)
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## Unreleased

## [0.16.4] - 2022-04-14

### Added

- Add `PyTzInfoAccess` trait for safe access to time zone information. [#2263](https://github.com/PyO3/pyo3/pull/2263)
- Add an experimental `generate-abi3-import-lib` feature to auto-generate `python3.dll` import libraries for Windows. [#2282](https://github.com/PyO3/pyo3/pull/2282)
- Add FFI definitions for `PyDateTime_BaseTime` and `PyDateTime_BaseDateTime`. [#2294](https://github.com/PyO3/pyo3/pull/2294)

### Changed

- Improved performance of failing calls to `FromPyObject::extract` which is common when functions accept multiple distinct types. [#2279](https://github.com/PyO3/pyo3/pull/2279)
- Default to "m" ABI tag when choosing `libpython` link name for CPython 3.7 on Unix. [#2288](https://github.com/PyO3/pyo3/pull/2288)
- Allow to compile "abi3" extensions without a working build host Python interpreter. [#2293](https://github.com/PyO3/pyo3/pull/2293)

### Fixed

- Crates depending on PyO3 can collect code coverage via LLVM instrumentation using stable Rust. [#2286](https://github.com/PyO3/pyo3/pull/2286)
- Fix segfault when calling FFI methods `PyDateTime_DATE_GET_TZINFO` or `PyDateTime_TIME_GET_TZINFO` on `datetime` or `time` without a tzinfo. [#2289](https://github.com/PyO3/pyo3/pull/2289)
- Fix directory names starting with the letter `n` breaking serialization of the interpreter configuration on Windows since PyO3 0.16.3. [#2299](https://github.com/PyO3/pyo3/pull/2299)

## [0.16.3] - 2022-04-05

### Packaging

- Extend `parking_lot` dependency supported versions to include 0.12. [#2239](https://github.com/PyO3/pyo3/pull/2239)

### Added

- Add methods to `pyo3_build_config::InterpreterConfig` to run Python scripts using the configured executable. [#2092](https://github.com/PyO3/pyo3/pull/2092)
- Add `as_bytes` method to `Py<PyBytes>`. [#2235](https://github.com/PyO3/pyo3/pull/2235)
- Add FFI definitions for `PyType_FromModuleAndSpec`, `PyType_GetModule`, `PyType_GetModuleState` and `PyModule_AddType`. [#2250](https://github.com/PyO3/pyo3/pull/2250)
- Add `pyo3_build_config::cross_compiling_from_to` as a helper to detect when PyO3 is cross-compiling. [#2253](https://github.com/PyO3/pyo3/pull/2253)
- Add `#[pyclass(mapping)]` option to leave sequence slots empty in container implementations. [#2265](https://github.com/PyO3/pyo3/pull/2265)
- Add `PyString::intern` to enable usage of the Python's built-in string interning. [#2268](https://github.com/PyO3/pyo3/pull/2268)
- Add `intern!` macro which can be used to amortize the cost of creating Python strings by storing them inside a `GILOnceCell`. [#2269](https://github.com/PyO3/pyo3/pull/2269)
- Add `PYO3_CROSS_PYTHON_IMPLEMENTATION` environment variable for selecting the default cross Python implementation. [#2272](https://github.com/PyO3/pyo3/pull/2272)

### Changed

- Allow `#[pyo3(crate = "...", text_signature = "...")]` options to be used directly in `#[pyclass(crate = "...", text_signature = "...")]`. [#2234](https://github.com/PyO3/pyo3/pull/2234)
- Make `PYO3_CROSS_LIB_DIR` environment variable optional when cross compiling. [#2241](https://github.com/PyO3/pyo3/pull/2241)
- Mark `METH_FASTCALL` calling convention as limited API on Python 3.10. [#2250](https://github.com/PyO3/pyo3/pull/2250)
- Deprecate `pyo3_build_config::cross_compiling` in favour of `pyo3_build_config::cross_compiling_from_to`. [#2253](https://github.com/PyO3/pyo3/pull/2253)

### Fixed

- Fix `abi3-py310` feature: use Python 3.10 ABI when available instead of silently falling back to the 3.9 ABI. [#2242](https://github.com/PyO3/pyo3/pull/2242)
- Use shared linking mode when cross compiling against a [Framework bundle](https://developer.apple.com/library/archive/documentation/MacOSX/Conceptual/BPFrameworks/Concepts/FrameworkAnatomy.html) for macOS. [#2233](https://github.com/PyO3/pyo3/pull/2233)
- Fix panic during compilation when `PYO3_CROSS_LIB_DIR` is set for some host/target combinations. [#2232](https://github.com/PyO3/pyo3/pull/2232)
- Correct dependency version for `syn` to require minimal patch version 1.0.56. [#2240](https://github.com/PyO3/pyo3/pull/2240)

## [0.16.2] - 2022-03-15

### Packaging

- Warn when modules are imported on PyPy 3.7 versions older than PyPy 7.3.8, as they are known to have binary compatibility issues. [#2217](https://github.com/PyO3/pyo3/pull/2217)
- Ensure build script of `pyo3-ffi` runs before that of `pyo3` to fix cross compilation. [#2224](https://github.com/PyO3/pyo3/pull/2224)

## [0.16.1] - 2022-03-05

### Packaging

- Extend `hashbrown` optional dependency supported versions to include 0.12. [#2197](https://github.com/PyO3/pyo3/pull/2197)

### Fixed

- Fix incorrect platform detection for Windows in `pyo3-build-config`. [#2198](https://github.com/PyO3/pyo3/pull/2198)
- Fix regression from 0.16 preventing cross compiling to aarch64 macOS. [#2201](https://github.com/PyO3/pyo3/pull/2201)

## [0.16.0] - 2022-02-27

### Packaging

- Update MSRV to Rust 1.48. [#2004](https://github.com/PyO3/pyo3/pull/2004)
- Update `indoc` optional dependency to 1.0. [#2004](https://github.com/PyO3/pyo3/pull/2004)
- Drop support for Python 3.6, remove `abi3-py36` feature. [#2006](https://github.com/PyO3/pyo3/pull/2006)
- `pyo3-build-config` no longer enables the `resolve-config` feature by default. [#2008](https://github.com/PyO3/pyo3/pull/2008)
- Update `inventory` optional dependency to 0.2. [#2019](https://github.com/PyO3/pyo3/pull/2019)
- Drop `paste` dependency. [#2081](https://github.com/PyO3/pyo3/pull/2081)
- The bindings found in `pyo3::ffi` are now a re-export of a separate `pyo3-ffi` crate. [#2126](https://github.com/PyO3/pyo3/pull/2126)
- Support PyPy 3.9. [#2143](https://github.com/PyO3/pyo3/pull/2143)

### Added

- Add `PyCapsule` type exposing the [Capsule API](https://docs.python.org/3/c-api/capsule.html#capsules). [#1980](https://github.com/PyO3/pyo3/pull/1980)
- Add `pyo3_build_config::Sysconfigdata` and supporting APIs. [#1996](https://github.com/PyO3/pyo3/pull/1996)
- Add `Py::setattr` method. [#2009](https://github.com/PyO3/pyo3/pull/2009)
- Add `#[pyo3(crate = "some::path")]` option to all attribute macros (except the deprecated `#[pyproto]`). [#2022](https://github.com/PyO3/pyo3/pull/2022)
- Enable `create_exception!` macro to take an optional docstring. [#2027](https://github.com/PyO3/pyo3/pull/2027)
- Enable `#[pyclass]` for fieldless (aka C-like) enums. [#2034](https://github.com/PyO3/pyo3/pull/2034)
- Add buffer magic methods `__getbuffer__` and `__releasebuffer__` to `#[pymethods]`. [#2067](https://github.com/PyO3/pyo3/pull/2067)
- Add support for paths in `wrap_pyfunction` and `wrap_pymodule`. [#2081](https://github.com/PyO3/pyo3/pull/2081)
- Enable `wrap_pyfunction!` to wrap a `#[pyfunction]` implemented in a different Rust module or crate. [#2091](https://github.com/PyO3/pyo3/pull/2091)
- Add `PyAny::contains` method (`in` operator for `PyAny`). [#2115](https://github.com/PyO3/pyo3/pull/2115)
- Add `PyMapping::contains` method (`in` operator for `PyMapping`). [#2133](https://github.com/PyO3/pyo3/pull/2133)
- Add garbage collection magic magic methods `__traverse__` and `__clear__` to `#[pymethods]`. [#2159](https://github.com/PyO3/pyo3/pull/2159)
- Add support for `from_py_with` on struct tuples and enums to override the default from-Python conversion. [#2181](https://github.com/PyO3/pyo3/pull/2181)
- Add `eq`, `ne`, `lt`, `le`, `gt`, `ge` methods to `PyAny` that wrap `rich_compare`. [#2175](https://github.com/PyO3/pyo3/pull/2175)
- Add `Py::is` and `PyAny::is` methods to check for object identity. [#2183](https://github.com/PyO3/pyo3/pull/2183)
- Add support for the `__getattribute__` magic method. [#2187](https://github.com/PyO3/pyo3/pull/2187)

### Changed

- `PyType::is_subclass`, `PyErr::is_instance` and `PyAny::is_instance` now operate run-time type object instead of a type known at compile-time. The old behavior is still available as `PyType::is_subclass_of`, `PyErr::is_instance_of` and `PyAny::is_instance_of`.  [#1985](https://github.com/PyO3/pyo3/pull/1985)
- Rename some methods on `PyErr` (the old names are just marked deprecated for now): [#2026](https://github.com/PyO3/pyo3/pull/2026)
  - `pytype` -> `get_type`
  - `pvalue` -> `value` (and deprecate equivalent `instance`)
  - `ptraceback` -> `traceback`
  - `from_instance` -> `from_value`
  - `into_instance` -> `into_value`
- `PyErr::new_type` now takes an optional docstring and now returns `PyResult<Py<PyType>>` rather than a `ffi::PyTypeObject` pointer. [#2027](https://github.com/PyO3/pyo3/pull/2027)
- Deprecate `PyType::is_instance`; it is inconsistent with other `is_instance` methods in PyO3. Instead of `typ.is_instance(obj)`, use `obj.is_instance(typ)`. [#2031](https://github.com/PyO3/pyo3/pull/2031)
- `__getitem__`, `__setitem__` and `__delitem__` in `#[pymethods]` now implement both a Python mapping and sequence by default. [#2065](https://github.com/PyO3/pyo3/pull/2065)
- Improve performance and error messages for `#[derive(FromPyObject)]` for enums. [#2068](https://github.com/PyO3/pyo3/pull/2068)
- Reduce generated LLVM code size (to improve compile times) for:
  - internal `handle_panic` helper [#2074](https://github.com/PyO3/pyo3/pull/2074) [#2158](https://github.com/PyO3/pyo3/pull/2158)
  - `#[pyfunction]` and `#[pymethods]` argument extraction [#2075](https://github.com/PyO3/pyo3/pull/2075) [#2085](https://github.com/PyO3/pyo3/pull/2085)
  - `#[pyclass]` type object creation [#2076](https://github.com/PyO3/pyo3/pull/2076) [#2081](https://github.com/PyO3/pyo3/pull/2081) [#2157](https://github.com/PyO3/pyo3/pull/2157)
- Respect Rust privacy rules for items wrapped with `wrap_pyfunction` and `wrap_pymodule`. [#2081](https://github.com/PyO3/pyo3/pull/2081)
- Add modulo argument to `__ipow__` magic method. [#2083](https://github.com/PyO3/pyo3/pull/2083)
- Fix FFI definition for `_PyCFunctionFast`. [#2126](https://github.com/PyO3/pyo3/pull/2126)
- `PyDateTimeAPI` and `PyDateTime_TimeZone_UTC` are are now unsafe functions instead of statics. [#2126](https://github.com/PyO3/pyo3/pull/2126)
- `PyDateTimeAPI` does not implicitly call `PyDateTime_IMPORT` anymore to reflect the original Python API more closely. Before the first call to `PyDateTime_IMPORT` a null pointer is returned. Therefore before calling any of the following FFI functions `PyDateTime_IMPORT` must be called to avoid undefined behaviour: [#2126](https://github.com/PyO3/pyo3/pull/2126)
  - `PyDateTime_TimeZone_UTC`
  - `PyDate_Check`
  - `PyDate_CheckExact`
  - `PyDateTime_Check`
  - `PyDateTime_CheckExact`
  - `PyTime_Check`
  - `PyTime_CheckExact`
  - `PyDelta_Check`
  - `PyDelta_CheckExact`
  - `PyTZInfo_Check`
  - `PyTZInfo_CheckExact`
  - `PyDateTime_FromTimestamp`
  - `PyDate_FromTimestamp`
- Deprecate the `gc` option for `pyclass` (e.g. `#[pyclass(gc)]`). Just implement a `__traverse__` `#[pymethod]`. [#2159](https://github.com/PyO3/pyo3/pull/2159)
- The `ml_meth` field of `PyMethodDef` is now represented by the `PyMethodDefPointer` union. [2166](https://github.com/PyO3/pyo3/pull/2166)
- Deprecate the `#[pyproto]` traits. [#2173](https://github.com/PyO3/pyo3/pull/2173)

### Removed

- Remove all functionality deprecated in PyO3 0.14. [#2007](https://github.com/PyO3/pyo3/pull/2007)
- Remove `Default` impl for `PyMethodDef`. [#2166](https://github.com/PyO3/pyo3/pull/2166)
- Remove `PartialEq` impl for `Py` and `PyAny` (use the new `is()` instead). [#2183](https://github.com/PyO3/pyo3/pull/2183)

### Fixed

- Fix undefined symbol for `PyObject_HasAttr` on PyPy. [#2025](https://github.com/PyO3/pyo3/pull/2025)
- Fix memory leak in `PyErr::into_value`. [#2026](https://github.com/PyO3/pyo3/pull/2026)
- Fix clippy warning `needless-option-as-deref` in code generated by `#[pyfunction]` and `#[pymethods]`. [#2040](https://github.com/PyO3/pyo3/pull/2040)
- Fix undefined behavior in `PySlice::indices`. [#2061](https://github.com/PyO3/pyo3/pull/2061)
- Fix the `wrap_pymodule!` macro using the wrong name for a `#[pymodule]` with a `#[pyo3(name = "..")]` attribute. [#2081](https://github.com/PyO3/pyo3/pull/2081)
- Fix magic methods in `#[pymethods]` accepting implementations with the wrong number of arguments. [#2083](https://github.com/PyO3/pyo3/pull/2083)
- Fix panic in `#[pyfunction]` generated code when a required argument following an `Option` was not provided.  [#2093](https://github.com/PyO3/pyo3/pull/2093)
- Fixed undefined behaviour caused by incorrect `ExactSizeIterator` implementations. [#2124](https://github.com/PyO3/pyo3/pull/2124)
- Fix missing FFI definition `PyCMethod_New` on Python 3.9 and up. [#2143](https://github.com/PyO3/pyo3/pull/2143)
- Add missing FFI definitions `_PyLong_NumBits` and `_PyLong_AsByteArray` on PyPy. [#2146](https://github.com/PyO3/pyo3/pull/2146)
- Fix memory leak in implementation of `AsPyPointer` for `Option<T>`. [#2160](https://github.com/PyO3/pyo3/pull/2160)
- Fix FFI definition of `_PyLong_NumBits` to return `size_t` instead of `c_int`. [#2161](https://github.com/PyO3/pyo3/pull/2161)
- Fix `TypeError` thrown when argument parsing failed missing the originating causes. [2177](https://github.com/PyO3/pyo3/pull/2178)

## [0.15.2] - 2022-04-14

### Packaging

- Backport of PyPy 3.9 support from PyO3 0.16. [#2262](https://github.com/PyO3/pyo3/pull/2262)

## [0.15.1] - 2021-11-19

### Added

- Add implementations for `Py::as_ref()` and `Py::into_ref()` for `Py<PySequence>`, `Py<PyIterator>` and `Py<PyMapping>`. [#1682](https://github.com/PyO3/pyo3/pull/1682)
- Add `PyTraceback` type to represent and format Python tracebacks. [#1977](https://github.com/PyO3/pyo3/pull/1977)

### Changed

- `#[classattr]` constants with a known magic method name (which is lowercase) no longer trigger lint warnings expecting constants to be uppercase. [#1969](https://github.com/PyO3/pyo3/pull/1969)

### Fixed

- Fix creating `#[classattr]` by functions with the name of a known magic method. [#1969](https://github.com/PyO3/pyo3/pull/1969)
- Fix use of `catch_unwind` in `allow_threads` which can cause fatal crashes. [#1989](https://github.com/PyO3/pyo3/pull/1989)
- Fix build failure on PyPy when abi3 features are activated. [#1991](https://github.com/PyO3/pyo3/pull/1991)
- Fix mingw platform detection. [#1993](https://github.com/PyO3/pyo3/pull/1993)
- Fix panic in `__get__` implementation when accessing descriptor on type object. [#1997](https://github.com/PyO3/pyo3/pull/1997)

## [0.15.0] - 2021-11-03

### Packaging

- `pyo3`'s `Cargo.toml` now advertises `links = "python"` to inform Cargo that it links against *libpython*. [#1819](https://github.com/PyO3/pyo3/pull/1819)
- Added optional `anyhow` feature to convert `anyhow::Error` into `PyErr`. [#1822](https://github.com/PyO3/pyo3/pull/1822)
- Support Python 3.10. [#1889](https://github.com/PyO3/pyo3/pull/1889)
- Added optional `eyre` feature to convert `eyre::Report` into `PyErr`. [#1893](https://github.com/PyO3/pyo3/pull/1893)
- Support PyPy 3.8. [#1948](https://github.com/PyO3/pyo3/pull/1948)

### Added

- Add `PyList::get_item_unchecked` and `PyTuple::get_item_unchecked` to get items without bounds checks. [#1733](https://github.com/PyO3/pyo3/pull/1733)
- Support `#[doc = include_str!(...)]` attributes on Rust 1.54 and up. [#1746](https://github.com/PyO3/pyo3/issues/1746)
- Add `PyAny::py` as a convenience for `PyNativeType::py`. [#1751](https://github.com/PyO3/pyo3/pull/1751)
- Add implementation of `std::ops::Index<usize>` for `PyList`, `PyTuple` and `PySequence`. [#1825](https://github.com/PyO3/pyo3/pull/1825)
- Add range indexing implementations of `std::ops::Index` for `PyList`, `PyTuple` and `PySequence`. [#1829](https://github.com/PyO3/pyo3/pull/1829)
- Add `PyMapping` type to represent the Python mapping protocol. [#1844](https://github.com/PyO3/pyo3/pull/1844)
- Add commonly-used sequence methods to `PyList` and `PyTuple`. [#1849](https://github.com/PyO3/pyo3/pull/1849)
- Add `as_sequence` methods to `PyList` and `PyTuple`. [#1860](https://github.com/PyO3/pyo3/pull/1860)
- Add support for magic methods in `#[pymethods]`, intended as a replacement for `#[pyproto]`. [#1864](https://github.com/PyO3/pyo3/pull/1864)
- Add `abi3-py310` feature. [#1889](https://github.com/PyO3/pyo3/pull/1889)
- Add `PyCFunction::new_closure` to create a Python function from a Rust closure. [#1901](https://github.com/PyO3/pyo3/pull/1901)
- Add support for positional-only arguments in `#[pyfunction]`. [#1925](https://github.com/PyO3/pyo3/pull/1925)
- Add `PyErr::take` to attempt to fetch a Python exception if present. [#1957](https://github.com/PyO3/pyo3/pull/1957)

### Changed

- `PyList`, `PyTuple` and `PySequence`'s APIs now accepts only `usize` indices instead of `isize`.
  [#1733](https://github.com/PyO3/pyo3/pull/1733), [#1802](https://github.com/PyO3/pyo3/pull/1802),
  [#1803](https://github.com/PyO3/pyo3/pull/1803)
- `PyList::get_item` and `PyTuple::get_item` now return `PyResult<&PyAny>` instead of panicking. [#1733](https://github.com/PyO3/pyo3/pull/1733)
- `PySequence::in_place_repeat` and `PySequence::in_place_concat` now return `PyResult<&PySequence>` instead of `PyResult<()>`, which is needed in case of immutable sequences such as tuples. [#1803](https://github.com/PyO3/pyo3/pull/1803)
- `PySequence::get_slice` now returns `PyResult<&PySequence>` instead of `PyResult<&PyAny>`. [#1829](https://github.com/PyO3/pyo3/pull/1829)
- Deprecate `PyTuple::split_from`. [#1804](https://github.com/PyO3/pyo3/pull/1804)
- Deprecate `PyTuple::slice`, new method `PyTuple::get_slice` added with `usize` indices. [#1828](https://github.com/PyO3/pyo3/pull/1828)
- Deprecate FFI definitions `PyParser_SimpleParseStringFlags`, `PyParser_SimpleParseStringFlagsFilename`, `PyParser_SimpleParseFileFlags` when building for Python 3.9. [#1830](https://github.com/PyO3/pyo3/pull/1830)
- Mark FFI definitions removed in Python 3.10 `PyParser_ASTFromString`, `PyParser_ASTFromStringObject`, `PyParser_ASTFromFile`, `PyParser_ASTFromFileObject`, `PyParser_SimpleParseStringFlags`, `PyParser_SimpleParseStringFlagsFilename`, `PyParser_SimpleParseFileFlags`, `PyParser_SimpleParseString`, `PyParser_SimpleParseFile`, `Py_SymtableString`, and `Py_SymtableStringObject`. [#1830](https://github.com/PyO3/pyo3/pull/1830)
- `#[pymethods]` now handles magic methods similarly to `#[pyproto]`. In the future, `#[pyproto]` may be deprecated. [#1864](https://github.com/PyO3/pyo3/pull/1864)
- Deprecate FFI definitions `PySys_AddWarnOption`, `PySys_AddWarnOptionUnicode` and `PySys_HasWarnOptions`. [#1887](https://github.com/PyO3/pyo3/pull/1887)
- Deprecate `#[call]` attribute in favor of using `fn __call__`. [#1929](https://github.com/PyO3/pyo3/pull/1929)
- Fix missing FFI definition `_PyImport_FindExtensionObject` on Python 3.10. [#1942](https://github.com/PyO3/pyo3/pull/1942)
- Change `PyErr::fetch` to panic in debug mode if no exception is present. [#1957](https://github.com/PyO3/pyo3/pull/1957)

### Fixed

- Fix building with a conda environment on Windows. [#1873](https://github.com/PyO3/pyo3/pull/1873)
- Fix panic on Python 3.6 when calling `Python::with_gil` with Python initialized but threading not initialized. [#1874](https://github.com/PyO3/pyo3/pull/1874)
- Fix incorrect linking to version-specific DLL instead of `python3.dll` when cross-compiling to Windows with `abi3`. [#1880](https://github.com/PyO3/pyo3/pull/1880)
- Fix FFI definition for `PyTuple_ClearFreeList` incorrectly being present for Python 3.9 and up. [#1887](https://github.com/PyO3/pyo3/pull/1887)
- Fix panic in generated `#[derive(FromPyObject)]` for enums. [#1888](https://github.com/PyO3/pyo3/pull/1888)
- Fix cross-compiling to Python 3.7 builds with the "m" abi flag. [#1908](https://github.com/PyO3/pyo3/pull/1908)
- Fix `__mod__` magic method fallback to `__rmod__`. [#1934](https://github.com/PyO3/pyo3/pull/1934).
- Fix missing FFI definition `_PyImport_FindExtensionObject` on Python 3.10. [#1942](https://github.com/PyO3/pyo3/pull/1942)

## [0.14.5] - 2021-09-05

### Added

- Make `pyo3_build_config::InterpreterConfig` and subfields public. [#1848](https://github.com/PyO3/pyo3/pull/1848)
- Add `resolve-config` feature to the `pyo3-build-config` to control whether its build script does anything. [#1856](https://github.com/PyO3/pyo3/pull/1856)

### Fixed

- Fix 0.14.4 compile regression on `s390x-unknown-linux-gnu` target. [#1850](https://github.com/PyO3/pyo3/pull/1850)

## [0.14.4] - 2021-08-29

### Changed

- Mark `PyString::data` as `unsafe` and disable it and some supporting PyUnicode FFI APIs (which depend on a C bitfield) on big-endian targets. [#1834](https://github.com/PyO3/pyo3/pull/1834)

## [0.14.3] - 2021-08-22

### Added

- Add `PyString::data` to access the raw bytes stored in a Python string. [#1794](https://github.com/PyO3/pyo3/pull/1794)

### Fixed

- Raise `AttributeError` to avoid panic when calling `del` on a `#[setter]` defined class property. [#1779](https://github.com/PyO3/pyo3/pull/1779)
- Restrict FFI definitions `PyGILState_Check` and `Py_tracefunc` to the unlimited API. [#1787](https://github.com/PyO3/pyo3/pull/1787)
- Add missing `_type` field to `PyStatus` struct definition. [#1791](https://github.com/PyO3/pyo3/pull/1791)
- Reduce lower bound `num-complex` optional dependency to support interop with `rust-numpy` and `ndarray` when building with the MSRV of 1.41 [#1799](https://github.com/PyO3/pyo3/pull/1799)
- Fix memory leak in `Python::run_code`. [#1806](https://github.com/PyO3/pyo3/pull/1806)
- Fix memory leak in `PyModule::from_code`. [#1810](https://github.com/PyO3/pyo3/pull/1810)
- Remove use of `pyo3::` in `pyo3::types::datetime` which broke builds using `-Z avoid-dev-deps` [#1811](https://github.com/PyO3/pyo3/pull/1811)

## [0.14.2] - 2021-08-09

### Added

- Add `indexmap` feature to add `ToPyObject`, `IntoPy` and `FromPyObject` implementations for `indexmap::IndexMap`. [#1728](https://github.com/PyO3/pyo3/pull/1728)
- Add `pyo3_build_config::add_extension_module_link_args` to use in build scripts to set linker arguments (for macOS). [#1755](https://github.com/PyO3/pyo3/pull/1755)
- Add `Python::with_gil_unchecked` unsafe variation of `Python::with_gil` to allow obtaining a `Python` in scenarios where `Python::with_gil` would fail. [#1769](https://github.com/PyO3/pyo3/pull/1769)

### Changed

- `PyErr::new` no longer acquires the Python GIL internally. [#1724](https://github.com/PyO3/pyo3/pull/1724)
- Reverted PyO3 0.14.0's use of `cargo:rustc-cdylib-link-arg` in its build script, as Cargo unintentionally allowed crates to pass linker args to downstream crates in this way. Projects supporting macOS may need to restore `.cargo/config.toml` files. [#1755](https://github.com/PyO3/pyo3/pull/1755)

### Fixed

- Fix regression in 0.14.0 rejecting usage of `#[doc(hidden)]` on structs and functions annotated with PyO3 macros. [#1722](https://github.com/PyO3/pyo3/pull/1722)
- Fix regression in 0.14.0 leading to incorrect code coverage being computed for `#[pyfunction]`s. [#1726](https://github.com/PyO3/pyo3/pull/1726)
- Fix incorrect FFI definition of `Py_Buffer` on PyPy. [#1737](https://github.com/PyO3/pyo3/pull/1737)
- Fix incorrect calculation of `dictoffset` on 32-bit Windows. [#1475](https://github.com/PyO3/pyo3/pull/1475)
- Fix regression in 0.13.2 leading to linking to incorrect Python library on Windows "gnu" targets. [#1759](https://github.com/PyO3/pyo3/pull/1759)
- Fix compiler warning: deny trailing semicolons in expression macro. [#1762](https://github.com/PyO3/pyo3/pull/1762)
- Fix incorrect FFI definition of `Py_DecodeLocale`. The 2nd argument is now `*mut Py_ssize_t` instead of `Py_ssize_t`. [#1766](https://github.com/PyO3/pyo3/pull/1766)

## [0.14.1] - 2021-07-04

### Added

- Implement `IntoPy<PyObject>` for `&PathBuf` and `&OsString`. [#1712](https://github.com/PyO3/pyo3/pull/1712)

### Fixed

- Fix crashes on PyPy due to incorrect definitions of `PyList_SET_ITEM`. [#1713](https://github.com/PyO3/pyo3/pull/1713)

## [0.14.0] - 2021-07-03

### Packaging

- Update `num-bigint` optional dependency to 0.4. [#1481](https://github.com/PyO3/pyo3/pull/1481)
- Update `num-complex` optional dependency to 0.4. [#1482](https://github.com/PyO3/pyo3/pull/1482)
- Extend `hashbrown` optional dependency supported versions to include 0.11. [#1496](https://github.com/PyO3/pyo3/pull/1496)
- Support PyPy 3.7. [#1538](https://github.com/PyO3/pyo3/pull/1538)

### Added

- Extend conversions for `[T; N]` to all `N` using const generics (on Rust 1.51 and up). [#1128](https://github.com/PyO3/pyo3/pull/1128)
- Add conversions between `OsStr`/ `OsString` and Python strings. [#1379](https://github.com/PyO3/pyo3/pull/1379)
- Add conversions between `Path`/ `PathBuf` and Python strings (and `pathlib.Path` objects). [#1379](https://github.com/PyO3/pyo3/pull/1379) [#1654](https://github.com/PyO3/pyo3/pull/1654)
- Add a new set of `#[pyo3(...)]` attributes to control various PyO3 macro functionality:
  - `#[pyo3(from_py_with = "...")]` function arguments and struct fields to override the default from-Python conversion. [#1411](https://github.com/PyO3/pyo3/pull/1411)
  - `#[pyo3(name = "...")]` for setting Python names. [#1567](https://github.com/PyO3/pyo3/pull/1567)
  - `#[pyo3(text_signature = "...")]` for setting text signature. [#1658](https://github.com/PyO3/pyo3/pull/1658)
- Add FFI definition `PyCFunction_CheckExact` for Python 3.9 and later. [#1425](https://github.com/PyO3/pyo3/pull/1425)
- Add FFI definition `Py_IS_TYPE`. [#1429](https://github.com/PyO3/pyo3/pull/1429)
- Add FFI definition `_Py_InitializeMain`. [#1473](https://github.com/PyO3/pyo3/pull/1473)
- Add FFI definitions from `cpython/import.h`.[#1475](https://github.com/PyO3/pyo3/pull/1475)
- Add tuple and unit struct support for `#[pyclass]` macro. [#1504](https://github.com/PyO3/pyo3/pull/1504)
- Add FFI definition `PyDateTime_TimeZone_UTC`. [#1572](https://github.com/PyO3/pyo3/pull/1572)
- Add support for `#[pyclass(extends=Exception)]`. [#1591](https://github.com/PyO3/pyo3/pull/1591)
- Add `PyErr::cause` and `PyErr::set_cause`. [#1679](https://github.com/PyO3/pyo3/pull/1679)
- Add FFI definitions from `cpython/pystate.h`. [#1687](https://github.com/PyO3/pyo3/pull/1687/)
- Add `wrap_pyfunction!` macro to `pyo3::prelude`. [#1695](https://github.com/PyO3/pyo3/pull/1695)

### Changed

- Allow only one `#[pymethods]` block per `#[pyclass]` by default, to remove the dependency on `inventory`. Add a `multiple-pymethods` feature to opt-in the original behavior and dependency on `inventory`. [#1457](https://github.com/PyO3/pyo3/pull/1457)
- Change `PyTimeAccess::get_fold` to return a `bool` instead of a `u8`. [#1397](https://github.com/PyO3/pyo3/pull/1397)
- Deprecate FFI definition `PyCFunction_Call` for Python 3.9 and up. [#1425](https://github.com/PyO3/pyo3/pull/1425)
- Deprecate FFI definition `PyModule_GetFilename`. [#1425](https://github.com/PyO3/pyo3/pull/1425)
- The `auto-initialize` feature is no longer enabled by default. [#1443](https://github.com/PyO3/pyo3/pull/1443)
- Change `PyCFunction::new` and `PyCFunction::new_with_keywords` to take `&'static str` arguments rather than implicitly copying (and leaking) them. [#1450](https://github.com/PyO3/pyo3/pull/1450)
- Deprecate `PyModule::call`, `PyModule::call0`, `PyModule::call1` and `PyModule::get`. [#1492](https://github.com/PyO3/pyo3/pull/1492)
- Add length information to `PyBufferError`s raised from `PyBuffer::copy_to_slice` and `PyBuffer::copy_from_slice`. [#1534](https://github.com/PyO3/pyo3/pull/1534)
- Automatically set `-undefined` and `dynamic_lookup` linker arguments on macOS with the `extension-module` feature. [#1539](https://github.com/PyO3/pyo3/pull/1539)
- Deprecate `#[pyproto]` methods which are easier to implement as `#[pymethods]`: [#1560](https://github.com/PyO3/pyo3/pull/1560)
  - `PyBasicProtocol::__bytes__` and `PyBasicProtocol::__format__`
  - `PyContextProtocol::__enter__` and `PyContextProtocol::__exit__`
  - `PyDescrProtocol::__delete__` and `PyDescrProtocol::__set_name__`
  - `PyMappingProtocol::__reversed__`
  - `PyNumberProtocol::__complex__` and `PyNumberProtocol::__round__`
  - `PyAsyncProtocol::__aenter__` and `PyAsyncProtocol::__aexit__`
- Deprecate several attributes in favor of the new `#[pyo3(...)]` options:
  - `#[name = "..."]`, replaced by `#[pyo3(name = "...")]` [#1567](https://github.com/PyO3/pyo3/pull/1567)
  - `#[pyfn(m, "name")]`, replaced by `#[pyfn(m)] #[pyo3(name = "...")]`. [#1610](https://github.com/PyO3/pyo3/pull/1610)
  - `#[pymodule(name)]`, replaced by `#[pymodule] #[pyo3(name = "...")]` [#1650](https://github.com/PyO3/pyo3/pull/1650)
  - `#[text_signature = "..."]`, replaced by `#[pyo3(text_signature = "...")]`. [#1658](https://github.com/PyO3/pyo3/pull/1658)
- Reduce LLVM line counts to improve compilation times. [#1604](https://github.com/PyO3/pyo3/pull/1604)
- No longer call `PyEval_InitThreads` in `#[pymodule]` init code. [#1630](https://github.com/PyO3/pyo3/pull/1630)
- Use `METH_FASTCALL` argument passing convention, when possible, to improve `#[pyfunction]` and method performance.
  [#1619](https://github.com/PyO3/pyo3/pull/1619), [#1660](https://github.com/PyO3/pyo3/pull/1660)
- Filter sysconfigdata candidates by architecture when cross-compiling. [#1626](https://github.com/PyO3/pyo3/pull/1626)

### Removed

- Remove deprecated exception names `BaseException` etc. [#1426](https://github.com/PyO3/pyo3/pull/1426)
- Remove deprecated methods `Python::is_instance`, `Python::is_subclass`, `Python::release`, `Python::xdecref`, and `Py::from_owned_ptr_or_panic`. [#1426](https://github.com/PyO3/pyo3/pull/1426)
- Remove many FFI definitions which never existed in the Python C-API:
  - (previously deprecated) `PyGetSetDef_INIT`, `PyGetSetDef_DICT`, `PyCoro_Check`, `PyCoroWrapper_Check`, and `PyAsyncGen_Check` [#1426](https://github.com/PyO3/pyo3/pull/1426)
  - `PyMethodDef_INIT` [#1426](https://github.com/PyO3/pyo3/pull/1426)
  - `PyTypeObject_INIT` [#1429](https://github.com/PyO3/pyo3/pull/1429)
  - `PyObject_Check`, `PySuper_Check`, and `FreeFunc` [#1438](https://github.com/PyO3/pyo3/pull/1438)
  - `PyModuleDef_INIT` [#1630](https://github.com/PyO3/pyo3/pull/1630)
- Remove pyclass implementation details from `PyTypeInfo`:
  - `Type`, `DESCRIPTION`, and `FLAGS` [#1456](https://github.com/PyO3/pyo3/pull/1456)
  - `BaseType`, `BaseLayout`, `Layout`, `Initializer` [#1596](https://github.com/PyO3/pyo3/pull/1596)
- Remove `PYO3_CROSS_INCLUDE_DIR` environment variable and the associated C header parsing functionality. [#1521](https://github.com/PyO3/pyo3/pull/1521)
- Remove `raw_pycfunction!` macro. [#1619](https://github.com/PyO3/pyo3/pull/1619)
- Remove `PyClassAlloc` trait. [#1657](https://github.com/PyO3/pyo3/pull/1657)
- Remove `PyList::get_parked_item`. [#1664](https://github.com/PyO3/pyo3/pull/1664)

### Fixed

- Remove FFI definition `PyCFunction_ClearFreeList` for Python 3.9 and later. [#1425](https://github.com/PyO3/pyo3/pull/1425)
- `PYO3_CROSS_LIB_DIR` enviroment variable no long required when compiling for x86-64 Python from macOS arm64 and reverse. [#1428](https://github.com/PyO3/pyo3/pull/1428)
- Fix FFI definition `_PyEval_RequestCodeExtraIndex`, which took an argument of the wrong type. [#1429](https://github.com/PyO3/pyo3/pull/1429)
- Fix FFI definition `PyIndex_Check` missing with the `abi3` feature. [#1436](https://github.com/PyO3/pyo3/pull/1436)
- Fix incorrect `TypeError` raised when keyword-only argument passed along with a positional argument in `*args`. [#1440](https://github.com/PyO3/pyo3/pull/1440)
- Fix inability to use a named lifetime for `&PyTuple` of `*args` in `#[pyfunction]`. [#1440](https://github.com/PyO3/pyo3/pull/1440)
- Fix use of Python argument for `#[pymethods]` inside macro expansions. [#1505](https://github.com/PyO3/pyo3/pull/1505)
- No longer include `__doc__` in `__all__` generated for `#[pymodule]`. [#1509](https://github.com/PyO3/pyo3/pull/1509)
- Always use cross-compiling configuration if any of the `PYO3_CROSS` family of environment variables are set. [#1514](https://github.com/PyO3/pyo3/pull/1514)
- Support `EnvironmentError`, `IOError`, and `WindowsError` on PyPy. [#1533](https://github.com/PyO3/pyo3/pull/1533)
- Fix unneccessary rebuilds when cycling between `cargo check` and `cargo clippy` in a Python virtualenv. [#1557](https://github.com/PyO3/pyo3/pull/1557)
- Fix segfault when dereferencing `ffi::PyDateTimeAPI` without the GIL. [#1563](https://github.com/PyO3/pyo3/pull/1563)
- Fix memory leak in `FromPyObject` implementations for `u128` and `i128`. [#1638](https://github.com/PyO3/pyo3/pull/1638)
- Fix `#[pyclass(extends=PyDict)]` leaking the dict contents on drop. [#1657](https://github.com/PyO3/pyo3/pull/1657)
- Fix segfault when calling `PyList::get_item` with negative indices. [#1668](https://github.com/PyO3/pyo3/pull/1668)
- Fix FFI definitions of `PyEval_SetProfile`/`PyEval_SetTrace` to take `Option<Py_tracefunc>` parameters. [#1692](https://github.com/PyO3/pyo3/pull/1692)
- Fix `ToPyObject` impl for `HashSet` to accept non-default hashers. [#1702](https://github.com/PyO3/pyo3/pull/1702)

## [0.13.2] - 2021-02-12

### Packaging

- Lower minimum supported Rust version to 1.41. [#1421](https://github.com/PyO3/pyo3/pull/1421)

### Added

- Add unsafe API `with_embedded_python_interpreter` to initialize a Python interpreter, execute a closure, and finalize the interpreter. [#1355](https://github.com/PyO3/pyo3/pull/1355)
- Add `serde` feature which provides implementations of `Serialize` and `Deserialize` for `Py<T>`. [#1366](https://github.com/PyO3/pyo3/pull/1366)
- Add FFI definition `_PyCFunctionFastWithKeywords` on Python 3.7 and up. [#1384](https://github.com/PyO3/pyo3/pull/1384)
- Add `PyDateTime::new_with_fold` method. [#1398](https://github.com/PyO3/pyo3/pull/1398)
- Add `size_hint` impls for `{PyDict,PyList,PySet,PyTuple}Iterator`s. [#1699](https://github.com/PyO3/pyo3/pull/1699)

### Changed

- `prepare_freethreaded_python` will no longer register an `atexit` handler to call `Py_Finalize`. This resolves a number of issues with incompatible C extensions causing crashes at finalization. [#1355](https://github.com/PyO3/pyo3/pull/1355)
- Mark `PyLayout::py_init`, `PyClassDict::clear_dict`, and `opt_to_pyobj` safe, as they do not perform any unsafe operations. [#1404](https://github.com/PyO3/pyo3/pull/1404)

### Fixed

- Fix support for using `r#raw_idents` as argument names in pyfunctions. [#1383](https://github.com/PyO3/pyo3/pull/1383)
- Fix typo in FFI definition for `PyFunction_GetCode` (was incorrectly `PyFunction_Code`). [#1387](https://github.com/PyO3/pyo3/pull/1387)
- Fix FFI definitions `PyMarshal_WriteObjectToString` and `PyMarshal_ReadObjectFromString` as available in limited API. [#1387](https://github.com/PyO3/pyo3/pull/1387)
- Fix FFI definitions `PyListObject` and those from `funcobject.h` as requiring non-limited API. [#1387](https://github.com/PyO3/pyo3/pull/1387)
- Fix unqualified `Result` usage in `pyobject_native_type_base`. [#1402](https://github.com/PyO3/pyo3/pull/1402)
- Fix build on systems where the default Python encoding is not UTF-8. [#1405](https://github.com/PyO3/pyo3/pull/1405)
- Fix build on mingw / MSYS2. [#1423](https://github.com/PyO3/pyo3/pull/1423)

## [0.13.1] - 2021-01-10

### Added

- Add support for `#[pyclass(dict)]` and `#[pyclass(weakref)]` with the `abi3` feature on Python 3.9 and up. [#1342](https://github.com/PyO3/pyo3/pull/1342)
- Add FFI definitions `PyOS_BeforeFork`, `PyOS_AfterFork_Parent`, `PyOS_AfterFork_Child` for Python 3.7 and up. [#1348](https://github.com/PyO3/pyo3/pull/1348)
- Add an `auto-initialize` feature to control whether PyO3 should automatically initialize an embedded Python interpreter. For compatibility this feature is enabled by default in PyO3 0.13.1, but is planned to become opt-in from PyO3 0.14.0. [#1347](https://github.com/PyO3/pyo3/pull/1347)
- Add support for cross-compiling to Windows without needing `PYO3_CROSS_INCLUDE_DIR`. [#1350](https://github.com/PyO3/pyo3/pull/1350)

### Deprecated

- Deprecate FFI definitions `PyEval_CallObjectWithKeywords`, `PyEval_CallObject`, `PyEval_CallFunction`, `PyEval_CallMethod` when building for Python 3.9. [#1338](https://github.com/PyO3/pyo3/pull/1338)
- Deprecate FFI definitions `PyGetSetDef_DICT` and `PyGetSetDef_INIT` which have never been in the Python API. [#1341](https://github.com/PyO3/pyo3/pull/1341)
- Deprecate FFI definitions `PyGen_NeedsFinalizing`, `PyImport_Cleanup` (removed in 3.9), and `PyOS_InitInterrupts` (3.10). [#1348](https://github.com/PyO3/pyo3/pull/1348)
- Deprecate FFI definition `PyOS_AfterFork` for Python 3.7 and up. [#1348](https://github.com/PyO3/pyo3/pull/1348)
- Deprecate FFI definitions `PyCoro_Check`, `PyAsyncGen_Check`, and `PyCoroWrapper_Check`, which have never been in the Python API (for the first two, it is possible to use `PyCoro_CheckExact` and `PyAsyncGen_CheckExact` instead; these are the actual functions provided by the Python API). [#1348](https://github.com/PyO3/pyo3/pull/1348)
- Deprecate FFI definitions for `PyUnicode_FromUnicode`, `PyUnicode_AsUnicode` and `PyUnicode_AsUnicodeAndSize`, which will be removed from 3.12 and up due to [PEP 613](https://www.python.org/dev/peps/pep-0623/). [#1370](https://github.com/PyO3/pyo3/pull/1370)

### Removed

- Remove FFI definition `PyFrame_ClearFreeList` when building for Python 3.9. [#1341](https://github.com/PyO3/pyo3/pull/1341)
- Remove FFI definition `_PyDict_Contains` when building for Python 3.10. [#1341](https://github.com/PyO3/pyo3/pull/1341)
- Remove FFI definitions `PyGen_NeedsFinalizing` and `PyImport_Cleanup` (for 3.9 and up), and `PyOS_InitInterrupts` (3.10). [#1348](https://github.com/PyO3/pyo3/pull/1348)

### Fixed

- Stop including `Py_TRACE_REFS` config setting automatically if `Py_DEBUG` is set on Python 3.8 and up. [#1334](https://github.com/PyO3/pyo3/pull/1334)
- Remove `#[deny(warnings)]` attribute (and instead refuse warnings only in CI). [#1340](https://github.com/PyO3/pyo3/pull/1340)
- Fix deprecation warning for missing `__module__` with `#[pyclass]`. [#1343](https://github.com/PyO3/pyo3/pull/1343)
- Correct return type of `PyFrozenSet::empty` to `&PyFrozenSet` (was incorrectly `&PySet`). [#1351](https://github.com/PyO3/pyo3/pull/1351)
- Fix missing `Py_INCREF` on heap type objects on Python versions before 3.8. [#1365](https://github.com/PyO3/pyo3/pull/1365)

## [0.13.0] - 2020-12-22

### Packaging

- Drop support for Python 3.5 (as it is now end-of-life). [#1250](https://github.com/PyO3/pyo3/pull/1250)
- Bump minimum supported Rust version to 1.45. [#1272](https://github.com/PyO3/pyo3/pull/1272)
- Bump indoc dependency to 1.0. [#1272](https://github.com/PyO3/pyo3/pull/1272)
- Bump paste dependency to 1.0. [#1272](https://github.com/PyO3/pyo3/pull/1272)
- Rename internal crates `pyo3cls` and `pyo3-derive-backend` to `pyo3-macros` and `pyo3-macros-backend` respectively. [#1317](https://github.com/PyO3/pyo3/pull/1317)

### Added

- Add support for building for CPython limited API. Opting-in to the limited API enables a single extension wheel built with PyO3 to be installable on multiple Python versions. This required a few minor changes to runtime behaviour of of PyO3 `#[pyclass]` types. See the migration guide for full details. [#1152](https://github.com/PyO3/pyo3/pull/1152)
  - Add feature flags `abi3-py36`, `abi3-py37`, `abi3-py38` etc. to set the minimum Python version when using the limited API. [#1263](https://github.com/PyO3/pyo3/pull/1263)
- Add argument names to `TypeError` messages generated by pymethod wrappers. [#1212](https://github.com/PyO3/pyo3/pull/1212)
- Add FFI definitions for PEP 587 "Python Initialization Configuration". [#1247](https://github.com/PyO3/pyo3/pull/1247)
- Add FFI definitions for `PyEval_SetProfile` and `PyEval_SetTrace`. [#1255](https://github.com/PyO3/pyo3/pull/1255)
- Add FFI definitions for context.h functions (`PyContext_New`, etc). [#1259](https://github.com/PyO3/pyo3/pull/1259)
- Add `PyAny::is_instance` method. [#1276](https://github.com/PyO3/pyo3/pull/1276)
- Add support for conversion between `char` and `PyString`. [#1282](https://github.com/PyO3/pyo3/pull/1282)
- Add FFI definitions for `PyBuffer_SizeFromFormat`, `PyObject_LengthHint`, `PyObject_CallNoArgs`, `PyObject_CallOneArg`, `PyObject_CallMethodNoArgs`, `PyObject_CallMethodOneArg`, `PyObject_VectorcallDict`, and `PyObject_VectorcallMethod`. [#1287](https://github.com/PyO3/pyo3/pull/1287)
- Add conversions between `u128`/`i128` and `PyLong` for PyPy. [#1310](https://github.com/PyO3/pyo3/pull/1310)
- Add `Python::version` and `Python::version_info` to get the running interpreter version. [#1322](https://github.com/PyO3/pyo3/pull/1322)
- Add conversions for tuples of length 10, 11, and 12. [#1454](https://github.com/PyO3/pyo3/pull/1454)

### Changed

- Change return type of `PyType::name` from `Cow<str>` to `PyResult<&str>`. [#1152](https://github.com/PyO3/pyo3/pull/1152)
- `#[pyclass(subclass)]` is now required for subclassing from Rust (was previously just required for subclassing from Python). [#1152](https://github.com/PyO3/pyo3/pull/1152)
- Change `PyIterator` to be consistent with other native types: it is now used as `&PyIterator` instead of `PyIterator<'a>`. [#1176](https://github.com/PyO3/pyo3/pull/1176)
- Change formatting of `PyDowncastError` messages to be closer to Python's builtin error messages. [#1212](https://github.com/PyO3/pyo3/pull/1212)
- Change `Debug` and `Display` impls for `PyException` to be consistent with `PyAny`. [#1275](https://github.com/PyO3/pyo3/pull/1275)
- Change `Debug` impl of `PyErr` to output more helpful information (acquiring the GIL if necessary). [#1275](https://github.com/PyO3/pyo3/pull/1275)
- Rename `PyTypeInfo::is_instance` and `PyTypeInfo::is_exact_instance` to `PyTypeInfo::is_type_of` and `PyTypeInfo::is_exact_type_of`. [#1278](https://github.com/PyO3/pyo3/pull/1278)
- Optimize `PyAny::call0`, `Py::call0` and `PyAny::call_method0` and `Py::call_method0` on Python 3.9 and up. [#1287](https://github.com/PyO3/pyo3/pull/1285)
- Require double-quotes for pyclass name argument e.g `#[pyclass(name = "MyClass")]`. [#1303](https://github.com/PyO3/pyo3/pull/1303)

### Deprecated

- Deprecate `Python::is_instance`, `Python::is_subclass`, `Python::release`, and `Python::xdecref`. [#1292](https://github.com/PyO3/pyo3/pull/1292)

### Removed

- Remove deprecated ffi definitions `PyUnicode_AsUnicodeCopy`, `PyUnicode_GetMax`, `_Py_CheckRecursionLimit`, `PyObject_AsCharBuffer`, `PyObject_AsReadBuffer`, `PyObject_CheckReadBuffer` and `PyObject_AsWriteBuffer`, which will be removed in Python 3.10. [#1217](https://github.com/PyO3/pyo3/pull/1217)
- Remove unused `python3` feature. [#1235](https://github.com/PyO3/pyo3/pull/1235)

### Fixed

- Fix missing field in `PyCodeObject` struct (`co_posonlyargcount`) - caused invalid access to other fields in Python >3.7. [#1260](https://github.com/PyO3/pyo3/pull/1260)
- Fix building for `x86_64-unknown-linux-musl` target from `x86_64-unknown-linux-gnu` host. [#1267](https://github.com/PyO3/pyo3/pull/1267)
- Fix `#[text_signature]` interacting badly with rust `r#raw_identifiers`. [#1286](https://github.com/PyO3/pyo3/pull/1286)
- Fix FFI definitions for `PyObject_Vectorcall` and `PyVectorcall_Call`. [#1287](https://github.com/PyO3/pyo3/pull/1285)
- Fix building with Anaconda python inside a virtualenv. [#1290](https://github.com/PyO3/pyo3/pull/1290)
- Fix definition of opaque FFI types. [#1312](https://github.com/PyO3/pyo3/pull/1312)
- Fix using custom error type in pyclass `#[new]` methods. [#1319](https://github.com/PyO3/pyo3/pull/1319)

## [0.12.4] - 2020-11-28

### Fixed

- Fix reference count bug in implementation of `From<Py<T>>` for `PyObject`, a regression introduced in PyO3 0.12. [#1297](https://github.com/PyO3/pyo3/pull/1297)

## [0.12.3] - 2020-10-12

### Fixed

- Fix support for Rust versions 1.39 to 1.44, broken by an incorrect internal update to paste 1.0 which was done in PyO3 0.12.2. [#1234](https://github.com/PyO3/pyo3/pull/1234)

## [0.12.2] - 2020-10-12

### Added

- Add support for keyword-only arguments without default values in `#[pyfunction]`. [#1209](https://github.com/PyO3/pyo3/pull/1209)
- Add `Python::check_signals` as a safe a wrapper for `PyErr_CheckSignals`. [#1214](https://github.com/PyO3/pyo3/pull/1214)

### Fixed

- Fix invalid document for protocol methods. [#1169](https://github.com/PyO3/pyo3/pull/1169)
- Hide docs of PyO3 private implementation details in `pyo3::class::methods`. [#1169](https://github.com/PyO3/pyo3/pull/1169)
- Fix unnecessary rebuild on PATH changes when the python interpreter is provided by PYO3_PYTHON. [#1231](https://github.com/PyO3/pyo3/pull/1231)

## [0.12.1] - 2020-09-16

### Fixed

- Fix building for a 32-bit Python on 64-bit Windows with a 64-bit Rust toolchain. [#1179](https://github.com/PyO3/pyo3/pull/1179)
- Fix building on platforms where `c_char` is `u8`. [#1182](https://github.com/PyO3/pyo3/pull/1182)

## [0.12.0] - 2020-09-12

### Added

- Add FFI definitions `Py_FinalizeEx`, `PyOS_getsig`, and `PyOS_setsig`. [#1021](https://github.com/PyO3/pyo3/pull/1021)
- Add `PyString::to_str` for accessing `PyString` as `&str`. [#1023](https://github.com/PyO3/pyo3/pull/1023)
- Add `Python::with_gil` for executing a closure with the Python GIL. [#1037](https://github.com/PyO3/pyo3/pull/1037)
- Add type information to failures in `PyAny::downcast`. [#1050](https://github.com/PyO3/pyo3/pull/1050)
- Implement `Debug` for `PyIterator`. [#1051](https://github.com/PyO3/pyo3/pull/1051)
- Add `PyBytes::new_with` and `PyByteArray::new_with` for initialising `bytes` and `bytearray` objects using a closure. [#1074](https://github.com/PyO3/pyo3/pull/1074)
- Add `#[derive(FromPyObject)]` macro for enums and structs. [#1065](https://github.com/PyO3/pyo3/pull/1065)
- Add `Py::as_ref` and `Py::into_ref` for converting `Py<T>` to `&T`. [#1098](https://github.com/PyO3/pyo3/pull/1098)
- Add ability to return `Result` types other than `PyResult` from `#[pyfunction]`, `#[pymethod]` and `#[pyproto]` functions. [#1106](https://github.com/PyO3/pyo3/pull/1118).
- Implement `ToPyObject`, `IntoPy`, and `FromPyObject` for [hashbrown](https://crates.io/crates/hashbrown)'s `HashMap` and `HashSet` types (requires the `hashbrown` feature). [#1114](https://github.com/PyO3/pyo3/pull/1114)
- Add `#[pyfunction(pass_module)]` and `#[pyfn(pass_module)]` to pass the module object as the first function argument. [#1143](https://github.com/PyO3/pyo3/pull/1143)
- Add `PyModule::add_function` and `PyModule::add_submodule` as typed alternatives to `PyModule::add_wrapped`. [#1143](https://github.com/PyO3/pyo3/pull/1143)
- Add native `PyCFunction` and `PyFunction` types. [#1163](https://github.com/PyO3/pyo3/pull/1163)

### Changed

- Rework exception types: [#1024](https://github.com/PyO3/pyo3/pull/1024) [#1115](https://github.com/PyO3/pyo3/pull/1115)
  - Rename exception types from e.g. `RuntimeError` to `PyRuntimeError`. The old names continue to exist but are deprecated.
  - Exception objects are now accessible as `&T` or `Py<T>`, just like other Python-native types.
  - Rename `PyException::py_err` to `PyException::new_err`.
  - Rename `PyUnicodeDecodeErr::new_err` to `PyUnicodeDecodeErr::new`.
  - Remove `PyStopIteration::stop_iteration`.
- Require `T: Send` for the return value `T` of `Python::allow_threads`. [#1036](https://github.com/PyO3/pyo3/pull/1036)
- Rename `PYTHON_SYS_EXECUTABLE` to `PYO3_PYTHON`. The old name will continue to work (undocumented) but will be removed in a future release. [#1039](https://github.com/PyO3/pyo3/pull/1039)
- Remove `unsafe` from signature of `PyType::as_type_ptr`. [#1047](https://github.com/PyO3/pyo3/pull/1047)
- Change return type of `PyIterator::from_object` to `PyResult<PyIterator>` (was `Result<PyIterator, PyDowncastError>`). [#1051](https://github.com/PyO3/pyo3/pull/1051)
- `IntoPy` is no longer implied by `FromPy`. [#1063](https://github.com/PyO3/pyo3/pull/1063)
- Change `PyObject` to be a type alias for `Py<PyAny>`. [#1063](https://github.com/PyO3/pyo3/pull/1063)
- Rework `PyErr` to be compatible with the `std::error::Error` trait: [#1067](https://github.com/PyO3/pyo3/pull/1067) [#1115](https://github.com/PyO3/pyo3/pull/1115)
  - Implement `Display`, `Error`, `Send` and `Sync` for `PyErr` and `PyErrArguments`.
  - Add `PyErr::instance` for accessing `PyErr` as `&PyBaseException`.
  - `PyErr`'s fields are now an implementation detail. The equivalent values can be accessed with `PyErr::ptype`, `PyErr::pvalue` and `PyErr::ptraceback`.
  - Change receiver of `PyErr::print` and `PyErr::print_and_set_sys_last_vars` to `&self` (was `self`).
  - Remove `PyErrValue`, `PyErr::from_value`, `PyErr::into_normalized`, and `PyErr::normalize`.
  - Remove `PyException::into`.
  - Remove `Into<PyResult<T>>` for `PyErr` and `PyException`.
- Change methods generated by `#[pyproto]` to return `NotImplemented` if Python should try a reversed operation. #[1072](https://github.com/PyO3/pyo3/pull/1072)
- Change argument to `PyModule::add` to `impl IntoPy<PyObject>` (was `impl ToPyObject`). #[1124](https://github.com/PyO3/pyo3/pull/1124)

### Removed

- Remove many exception and `PyErr` APIs; see the "changed" section above. [#1024](https://github.com/PyO3/pyo3/pull/1024) [#1067](https://github.com/PyO3/pyo3/pull/1067) [#1115](https://github.com/PyO3/pyo3/pull/1115)
- Remove `PyString::to_string` (use new `PyString::to_str`). [#1023](https://github.com/PyO3/pyo3/pull/1023)
- Remove `PyString::as_bytes`. [#1023](https://github.com/PyO3/pyo3/pull/1023)
- Remove `Python::register_any`. [#1023](https://github.com/PyO3/pyo3/pull/1023)
- Remove `GILGuard::acquire` from the public API. Use `Python::acquire_gil` or `Python::with_gil`. [#1036](https://github.com/PyO3/pyo3/pull/1036)
- Remove the `FromPy` trait. [#1063](https://github.com/PyO3/pyo3/pull/1063)
- Remove the `AsPyRef` trait. [#1098](https://github.com/PyO3/pyo3/pull/1098)

### Fixed

- Correct FFI definitions `Py_SetProgramName` and `Py_SetPythonHome` to take `*const` arguments (was `*mut`). [#1021](https://github.com/PyO3/pyo3/pull/1021)
- Fix `FromPyObject` for `num_bigint::BigInt` for Python objects with an `__index__` method. [#1027](https://github.com/PyO3/pyo3/pull/1027)
- Correct FFI definition `_PyLong_AsByteArray` to take `*mut c_uchar` argument (was `*const c_uchar`). [#1029](https://github.com/PyO3/pyo3/pull/1029)
- Fix segfault with `#[pyclass(dict, unsendable)]`. [#1058](https://github.com/PyO3/pyo3/pull/1058) [#1059](https://github.com/PyO3/pyo3/pull/1059)
- Fix using `&Self` as an argument type for functions in a `#[pymethods]` block. [#1071](https://github.com/PyO3/pyo3/pull/1071)
- Fix best-effort build against PyPy 3.6. [#1092](https://github.com/PyO3/pyo3/pull/1092)
- Fix many cases of lifetime elision in `#[pyproto]` implementations. [#1093](https://github.com/PyO3/pyo3/pull/1093)
- Fix detection of Python build configuration when cross-compiling. [#1095](https://github.com/PyO3/pyo3/pull/1095)
- Always link against libpython on android with the `extension-module` feature. [#1095](https://github.com/PyO3/pyo3/pull/1095)
- Fix the `+` operator not trying `__radd__` when both `__add__` and `__radd__` are defined in `PyNumberProtocol` (and similar for all other reversible operators). [#1107](https://github.com/PyO3/pyo3/pull/1107)
- Fix building with Anaconda python. [#1175](https://github.com/PyO3/pyo3/pull/1175)

## [0.11.1] - 2020-06-30

### Added

- `#[pyclass(unsendable)]`. [#1009](https://github.com/PyO3/pyo3/pull/1009)

### Changed

- Update `parking_lot` dependency to `0.11`. [#1010](https://github.com/PyO3/pyo3/pull/1010)

## [0.11.0] - 2020-06-28

### Added

- Support stable versions of Rust (>=1.39). [#969](https://github.com/PyO3/pyo3/pull/969)
- Add FFI definition `PyObject_AsFileDescriptor`. [#938](https://github.com/PyO3/pyo3/pull/938)
- Add `PyByteArray::data`, `PyByteArray::as_bytes`, and `PyByteArray::as_bytes_mut`. [#967](https://github.com/PyO3/pyo3/pull/967)
- Add `GILOnceCell` to use in situations where `lazy_static` or `once_cell` can deadlock. [#975](https://github.com/PyO3/pyo3/pull/975)
- Add `Py::borrow`, `Py::borrow_mut`, `Py::try_borrow`, and `Py::try_borrow_mut` for accessing `#[pyclass]` values. [#976](https://github.com/PyO3/pyo3/pull/976)
- Add `IterNextOutput` and `IterANextOutput` for returning from `__next__` / `__anext__`. [#997](https://github.com/PyO3/pyo3/pull/997)

### Changed

- Simplify internals of `#[pyo3(get)]` attribute. (Remove the hidden API `GetPropertyValue`.) [#934](https://github.com/PyO3/pyo3/pull/934)
- Call `Py_Finalize` at exit to flush buffers, etc. [#943](https://github.com/PyO3/pyo3/pull/943)
- Add type parameter to PyBuffer. #[951](https://github.com/PyO3/pyo3/pull/951)
- Require `Send` bound for `#[pyclass]`. [#966](https://github.com/PyO3/pyo3/pull/966)
- Add `Python` argument to most methods on `PyObject` and `Py<T>` to ensure GIL safety. [#970](https://github.com/PyO3/pyo3/pull/970)
- Change signature of `PyTypeObject::type_object` - now takes `Python` argument and returns `&PyType`. [#970](https://github.com/PyO3/pyo3/pull/970)
- Change return type of `PyTuple::slice` and `PyTuple::split_from` from `Py<PyTuple>` to `&PyTuple`. [#970](https://github.com/PyO3/pyo3/pull/970)
- Change return type of `PyTuple::as_slice` to `&[&PyAny]`. [#971](https://github.com/PyO3/pyo3/pull/971)
- Rename `PyTypeInfo::type_object` to `type_object_raw`, and add `Python` argument. [#975](https://github.com/PyO3/pyo3/pull/975)
- Update `num-complex` optional dependendency from `0.2` to `0.3`. [#977](https://github.com/PyO3/pyo3/pull/977)
- Update `num-bigint` optional dependendency from `0.2` to `0.3`. [#978](https://github.com/PyO3/pyo3/pull/978)
- `#[pyproto]` is re-implemented without specialization. [#961](https://github.com/PyO3/pyo3/pull/961)
- `PyClassAlloc::alloc` is renamed to `PyClassAlloc::new`. [#990](https://github.com/PyO3/pyo3/pull/990)
- `#[pyproto]` methods can now have return value `T` or `PyResult<T>` (previously only `PyResult<T>` was supported). [#996](https://github.com/PyO3/pyo3/pull/996)
- `#[pyproto]` methods can now skip annotating the return type if it is `()`. [#998](https://github.com/PyO3/pyo3/pull/998)

### Removed

- Remove `ManagedPyRef` (unused, and needs specialization) [#930](https://github.com/PyO3/pyo3/pull/930)

### Fixed

- Fix passing explicit `None` to `Option<T>` argument `#[pyfunction]` with a default value. [#936](https://github.com/PyO3/pyo3/pull/936)
- Fix `PyClass.__new__`'s not respecting subclasses when inherited by a Python class. [#990](https://github.com/PyO3/pyo3/pull/990)
- Fix returning `Option<T>` from `#[pyproto]` methods. [#996](https://github.com/PyO3/pyo3/pull/996)
- Fix accepting `PyRef<Self>` and `PyRefMut<Self>` to `#[getter]` and `#[setter]` methods. [#999](https://github.com/PyO3/pyo3/pull/999)

## [0.10.1] - 2020-05-14

### Fixed

- Fix deadlock in `Python::acquire_gil` after dropping a `PyObject` or `Py<T>`. [#924](https://github.com/PyO3/pyo3/pull/924)

## [0.10.0] - 2020-05-13

### Added

- Add FFI definition `_PyDict_NewPresized`. [#849](https://github.com/PyO3/pyo3/pull/849)
- Implement `IntoPy<PyObject>` for `HashSet` and `BTreeSet`. [#864](https://github.com/PyO3/pyo3/pull/864)
- Add `PyAny::dir` method. [#886](https://github.com/PyO3/pyo3/pull/886)
- Gate macros behind a `macros` feature (enabled by default). [#897](https://github.com/PyO3/pyo3/pull/897)
- Add ability to define class attributes using `#[classattr]` on functions in `#[pymethods]`. [#905](https://github.com/PyO3/pyo3/pull/905)
- Implement `Clone` for `PyObject` and `Py<T>`. [#908](https://github.com/PyO3/pyo3/pull/908)
- Implement `Deref<Target = PyAny>` for all builtin types. (`PyList`, `PyTuple`, `PyDict` etc.) [#911](https://github.com/PyO3/pyo3/pull/911)
- Implement `Deref<Target = PyAny>` for `PyCell<T>`. [#911](https://github.com/PyO3/pyo3/pull/911)
- Add `#[classattr]` support for associated constants in `#[pymethods]`. [#914](https://github.com/PyO3/pyo3/pull/914)

### Changed

- Panics will now be raised as a Python `PanicException`. [#797](https://github.com/PyO3/pyo3/pull/797)
- Change `PyObject` and `Py<T>` reference counts to decrement immediately upon drop when the GIL is held. [#851](https://github.com/PyO3/pyo3/pull/851)
- Allow `PyIterProtocol` methods to use either `PyRef` or `PyRefMut` as the receiver type. [#856](https://github.com/PyO3/pyo3/pull/856)
- Change the implementation of `FromPyObject` for `Py<T>` to apply to a wider range of `T`, including all `T: PyClass`. [#880](https://github.com/PyO3/pyo3/pull/880)
- Move all methods from the `ObjectProtocol` trait to the `PyAny` struct. [#911](https://github.com/PyO3/pyo3/pull/911)
- Remove need for `#![feature(specialization)]` in crates depending on PyO3. [#917](https://github.com/PyO3/pyo3/pull/917)

### Removed

- Remove `PyMethodsProtocol` trait. [#889](https://github.com/PyO3/pyo3/pull/889)
- Remove `num-traits` dependency. [#895](https://github.com/PyO3/pyo3/pull/895)
- Remove `ObjectProtocol` trait. [#911](https://github.com/PyO3/pyo3/pull/911)
- Remove `PyAny::None`. Users should use `Python::None` instead. [#911](https://github.com/PyO3/pyo3/pull/911)
- Remove all `*ProtocolImpl` traits. [#917](https://github.com/PyO3/pyo3/pull/917)

### Fixed

- Fix support for `__radd__` and other `__r*__` methods as implementations for Python mathematical operators. [#839](https://github.com/PyO3/pyo3/pull/839)
- Fix panics during garbage collection when traversing objects that were already mutably borrowed. [#855](https://github.com/PyO3/pyo3/pull/855)
- Prevent `&'static` references to Python objects as arguments to `#[pyfunction]` and `#[pymethods]`. [#869](https://github.com/PyO3/pyo3/pull/869)
- Fix lifetime safety bug with `AsPyRef::as_ref`. [#876](https://github.com/PyO3/pyo3/pull/876)
- Fix `#[pyo3(get)]` attribute on `Py<T>` fields. [#880](https://github.com/PyO3/pyo3/pull/880)
- Fix segmentation faults caused by functions such as `PyList::get_item` returning borrowed objects when it was not safe to do so. [#890](https://github.com/PyO3/pyo3/pull/890)
- Fix segmentation faults caused by nested `Python::acquire_gil` calls creating dangling references. [#893](https://github.com/PyO3/pyo3/pull/893)
- Fix segmentatation faults when a panic occurs during a call to `Python::allow_threads`. [#912](https://github.com/PyO3/pyo3/pull/912)

## [0.9.2] - 2020-04-09

### Added

- `FromPyObject` implementations for `HashSet` and `BTreeSet`. [#842](https://github.com/PyO3/pyo3/pull/842)

### Fixed

- Correctly detect 32bit architecture. [#830](https://github.com/PyO3/pyo3/pull/830)

## [0.9.1] - 2020-03-23

### Fixed

- Error messages for `#[pyclass]`. [#826](https://github.com/PyO3/pyo3/pull/826)
- `FromPyObject` implementation for `PySequence`. [#827](https://github.com/PyO3/pyo3/pull/827)

## [0.9.0] - 2020-03-19

### Added

- `PyCell`, which has RefCell-like features. [#770](https://github.com/PyO3/pyo3/pull/770)
- `PyClass`, `PyLayout`, `PyClassInitializer`. [#683](https://github.com/PyO3/pyo3/pull/683)
- Implemented `IntoIterator` for `PySet` and `PyFrozenSet`. [#716](https://github.com/PyO3/pyo3/pull/716)
- `FromPyObject` is now automatically implemented for `T: Clone` pyclasses. [#730](https://github.com/PyO3/pyo3/pull/730)
- `#[pyo3(get)]` and `#[pyo3(set)]` will now use the Rust doc-comment from the field for the Python property. [#755](https://github.com/PyO3/pyo3/pull/755)
- `#[setter]` functions may now take an argument of `Pyo3::Python`. [#760](https://github.com/PyO3/pyo3/pull/760)
- `PyTypeInfo::BaseLayout` and `PyClass::BaseNativeType`. [#770](https://github.com/PyO3/pyo3/pull/770)
- `PyDowncastImpl`. [#770](https://github.com/PyO3/pyo3/pull/770)
- Implement `FromPyObject` and `IntoPy<PyObject>` traits for arrays (up to 32). [#778](https://github.com/PyO3/pyo3/pull/778)
- `migration.md` and `types.md` in the guide. [#795](https://github.com/PyO3/pyo3/pull/795), #[802](https://github.com/PyO3/pyo3/pull/802)
- `ffi::{_PyBytes_Resize, _PyDict_Next, _PyDict_Contains, _PyDict_GetDictPtr}`. #[820](https://github.com/PyO3/pyo3/pull/820)

### Changed

- `#[new]` does not take `PyRawObject` and can return `Self`. [#683](https://github.com/PyO3/pyo3/pull/683)
- The blanket implementations for `FromPyObject` for `&T` and `&mut T` are no longer specializable. Implement `PyTryFrom` for your type to control the behavior of `FromPyObject::extract` for your types. [#713](https://github.com/PyO3/pyo3/pull/713)
- The implementation for `IntoPy<U> for T` where `U: FromPy<T>` is no longer specializable. Control the behavior of this via the implementation of `FromPy`. [#713](https://github.com/PyO3/pyo3/pull/713)
- Use `parking_lot::Mutex` instead of `spin::Mutex`. [#734](https://github.com/PyO3/pyo3/pull/734)
- Bumped minimum Rust version to `1.42.0-nightly 2020-01-21`. [#761](https://github.com/PyO3/pyo3/pull/761)
- `PyRef` and `PyRefMut` are renewed for `PyCell`. [#770](https://github.com/PyO3/pyo3/pull/770)
- Some new FFI functions for Python 3.8. [#784](https://github.com/PyO3/pyo3/pull/784)
- `PyAny` is now on the top level module and prelude. [#816](https://github.com/PyO3/pyo3/pull/816)

### Removed

- `PyRawObject`. [#683](https://github.com/PyO3/pyo3/pull/683)
- `PyNoArgsFunction`. [#741](https://github.com/PyO3/pyo3/pull/741)
- `initialize_type`. To set the module name for a `#[pyclass]`, use the `module` argument to the macro. #[751](https://github.com/PyO3/pyo3/pull/751)
- `AsPyRef::as_mut/with/with_mut/into_py/into_mut_py`. [#770](https://github.com/PyO3/pyo3/pull/770)
- `PyTryFrom::try_from_mut/try_from_mut_exact/try_from_mut_unchecked`. [#770](https://github.com/PyO3/pyo3/pull/770)
- `Python::mut_from_owned_ptr/mut_from_borrowed_ptr`. [#770](https://github.com/PyO3/pyo3/pull/770)
- `ObjectProtocol::get_base/get_mut_base`. [#770](https://github.com/PyO3/pyo3/pull/770)

### Fixed

- Fixed unsoundness of subclassing. [#683](https://github.com/PyO3/pyo3/pull/683).
- Clear error indicator when the exception is handled on the Rust side. [#719](https://github.com/PyO3/pyo3/pull/719)
- Usage of raw identifiers with `#[pyo3(set)]`. [#745](https://github.com/PyO3/pyo3/pull/745)
- Usage of `PyObject` with `#[pyo3(get)]`. [#760](https://github.com/PyO3/pyo3/pull/760)
- `#[pymethods]` used in conjunction with `#[cfg]`. #[769](https://github.com/PyO3/pyo3/pull/769)
- `"*"` in a `#[pyfunction()]` argument list incorrectly accepting any number of positional arguments (use `args = "*"` when this behaviour is desired). #[792](https://github.com/PyO3/pyo3/pull/792)
- `PyModule::dict`. #[809](https://github.com/PyO3/pyo3/pull/809)
- Fix the case where `DESCRIPTION` is not null-terminated. #[822](https://github.com/PyO3/pyo3/pull/822)

## [0.8.5] - 2020-01-05

### Added

- Implemented `FromPyObject` for `HashMap` and `BTreeMap`
- Support for `#[name = "foo"]` attribute for `#[pyfunction]` and in `#[pymethods]`. [#692](https://github.com/PyO3/pyo3/pull/692)

## [0.8.4] - 2019-12-14

### Added

- Support for `#[text_signature]` attribute. [#675](https://github.com/PyO3/pyo3/pull/675)

## [0.8.3] - 2019-11-23

### Removed

- `#[init]` is removed. [#658](https://github.com/PyO3/pyo3/pull/658)

### Fixed

- Now all `&Py~` types have `!Send` bound. [#655](https://github.com/PyO3/pyo3/pull/655)
- Fix a compile error raised by the stabilization of `!` type. [#672](https://github.com/PyO3/pyo3/issues/672).

## [0.8.2] - 2019-10-27

### Added

- FFI compatibility for PEP 590 Vectorcall. [#641](https://github.com/PyO3/pyo3/pull/641)

### Fixed

- Fix PySequenceProtocol::set_item. [#624](https://github.com/PyO3/pyo3/pull/624)
- Fix a corner case of BigInt::FromPyObject. [#630](https://github.com/PyO3/pyo3/pull/630)
- Fix index errors in parameter conversion. [#631](https://github.com/PyO3/pyo3/pull/631)
- Fix handling of invalid utf-8 sequences in `PyString::as_bytes`. [#639](https://github.com/PyO3/pyo3/pull/639)
  and `PyString::to_string_lossy` [#642](https://github.com/PyO3/pyo3/pull/642).
- Remove `__contains__` and `__iter__` from PyMappingProtocol. [#644](https://github.com/PyO3/pyo3/pull/644)
- Fix proc-macro definition of PySetAttrProtocol. [#645](https://github.com/PyO3/pyo3/pull/645)

## [0.8.1] - 2019-10-08

### Added

- Conversion between [num-bigint](https://github.com/rust-num/num-bigint) and Python int. [#608](https://github.com/PyO3/pyo3/pull/608)

### Fixed

- Make sure the right Python interpreter is used in OSX builds. [#604](https://github.com/PyO3/pyo3/pull/604)
- Patch specialization being broken by Rust 1.40. [#614](https://github.com/PyO3/pyo3/issues/614)
- Fix a segfault around PyErr. [#597](https://github.com/PyO3/pyo3/pull/597)

## [0.8.0] - 2019-09-16

### Added

- `module` argument to `pyclass` macro. [#499](https://github.com/PyO3/pyo3/pull/499)
- `py_run!` macro [#512](https://github.com/PyO3/pyo3/pull/512)
- Use existing fields and methods before calling custom **getattr**. [#505](https://github.com/PyO3/pyo3/pull/505)
- `PyBytes` can now be indexed just like `Vec<u8>`
- Implement `IntoPy<PyObject>` for `PyRef` and `PyRefMut`.

### Changed

- Implementing the Using the `gc` parameter for `pyclass` (e.g. `#[pyclass(gc)]`) without implementing the `class::PyGCProtocol` trait is now a compile-time error. Failing to implement this trait could lead to segfaults. [#532](https://github.com/PyO3/pyo3/pull/532)
- `PyByteArray::data` has been replaced with `PyDataArray::to_vec` because returning a `&[u8]` is unsound. (See [this comment](https://github.com/PyO3/pyo3/issues/373#issuecomment-512332696) for a great write-up for why that was unsound)
- Replace `mashup` with `paste`.
- `GILPool` gained a `Python` marker to prevent it from being misused to release Python objects without the GIL held.

### Removed

- `IntoPyObject` was replaced with `IntoPy<PyObject>`
- `#[pyclass(subclass)]` is hidden a `unsound-subclass` feature because it's causing segmentation faults.

### Fixed

- More readable error message for generics in pyclass [#503](https://github.com/PyO3/pyo3/pull/503)

## [0.7.0] - 2019-05-26

### Added

- PyPy support by omerbenamram in [#393](https://github.com/PyO3/pyo3/pull/393)
- Have `PyModule` generate an index of its members (`__all__` list).
- Allow `slf: PyRef<T>` for pyclass(#419)
- Allow to use lifetime specifiers in `pymethods`
- Add `marshal` module. [#460](https://github.com/PyO3/pyo3/pull/460)

### Changed

- `Python::run` returns `PyResult<()>` instead of `PyResult<&PyAny>`.
- Methods decorated with `#[getter]` and `#[setter]` can now omit wrapping the
  result type in `PyResult` if they don't raise exceptions.

### Fixed

- `type_object::PyTypeObject` has been marked unsafe because breaking the contract `type_object::PyTypeObject::init_type` can lead to UB.
- Fixed automatic derive of `PySequenceProtocol` implementation in [#423](https://github.com/PyO3/pyo3/pull/423).
- Capitalization & better wording to README.md.
- Docstrings of properties is now properly set using the doc of the `#[getter]` method.
- Fixed issues with `pymethods` crashing on doc comments containing double quotes.
- `PySet::new` and `PyFrozenSet::new` now return `PyResult<&Py[Frozen]Set>`; exceptions are raised if
  the items are not hashable.
- Fixed building using `venv` on Windows.
- `PyTuple::new` now returns `&PyTuple` instead of `Py<PyTuple>`.
- Fixed several issues with argument parsing; notable, the `*args` and `**kwargs`
  tuple/dict now doesn't contain arguments that are otherwise assigned to parameters.

## [0.6.0] - 2019-03-28

### Regressions

- Currently, [#341](https://github.com/PyO3/pyo3/issues/341) causes `cargo test` to fail with weird linking errors when the `extension-module` feature is activated. For now you can work around this by making the `extension-module` feature optional and running the tests with `cargo test --no-default-features`:

```toml
[dependencies.pyo3]
version = "0.6.0"

[features]
extension-module = ["pyo3/extension-module"]
default = ["extension-module"]
```

### Added

- Added a `wrap_pymodule!` macro similar to the existing `wrap_pyfunction!` macro. Only available on python 3
- Added support for cross compiling (e.g. to arm v7) by mtp401 in [#327](https://github.com/PyO3/pyo3/pull/327). See the "Cross Compiling" section in the "Building and Distribution" chapter of the guide for more details.
- The `PyRef` and `PyRefMut` types, which allow to differentiate between an instance of a rust struct on the rust heap and an instance that is embedded inside a python object. By kngwyu in [#335](https://github.com/PyO3/pyo3/pull/335)
- Added `FromPy<T>` and `IntoPy<T>` which are equivalent to `From<T>` and `Into<T>` except that they require a gil token.
- Added `ManagedPyRef`, which should eventually replace `ToBorrowedObject`.

### Changed

- Renamed `PyObjectRef` to `PyAny` in #388
- Renamed `add_function` to `add_wrapped` as it now also supports modules.
- Renamed `#[pymodinit]` to `#[pymodule]`
- `py.init(|| value)` becomes `Py::new(value)`
- `py.init_ref(|| value)` becomes `PyRef::new(value)`
- `py.init_mut(|| value)` becomes `PyRefMut::new(value)`.
- `PyRawObject::init` is now infallible, e.g. it returns `()` instead of `PyResult<()>`.
- Renamed `py_exception!` to `create_exception!` and refactored the error macros.
- Renamed `wrap_function!` to `wrap_pyfunction!`
- Renamed `#[prop(get, set)]` to `#[pyo3(get, set)]`
- `#[pyfunction]` now supports the same arguments as `#[pyfn()]`
- Some macros now emit proper spanned errors instead of panics.
- Migrated to the 2018 edition
- `crate::types::exceptions` moved to `crate::exceptions`
- Replace `IntoPyTuple` with `IntoPy<Py<PyTuple>>`.
- `IntoPyPointer` and `ToPyPointer` moved into the crate root.
- `class::CompareOp` moved into `class::basic::CompareOp`
- PyTypeObject is now a direct subtrait PyTypeCreate, removing the old cyclical implementation in [#350](https://github.com/PyO3/pyo3/pull/350)
- Add `PyList::{sort, reverse}` by chr1sj0nes in [#357](https://github.com/PyO3/pyo3/pull/357) and [#358](https://github.com/PyO3/pyo3/pull/358)
- Renamed the `typeob` module to `type_object`

### Removed

- `PyToken` was removed due to unsoundness (See [#94](https://github.com/PyO3/pyo3/issues/94)).
- Removed the unnecessary type parameter from `PyObjectAlloc`
- `NoArgs`. Just use an empty tuple
- `PyObjectWithGIL`. `PyNativeType` is sufficient now that PyToken is removed.

### Fixed

- A soudness hole where every instances of a `#[pyclass]` struct was considered to be part of a python object, even though you can create instances that are not part of the python heap. This was fixed through `PyRef` and `PyRefMut`.
- Fix kwargs support in [#328](https://github.com/PyO3/pyo3/pull/328).
- Add full support for `__dict__` in [#403](https://github.com/PyO3/pyo3/pull/403).

## [0.5.3] - 2019-01-04

### Fixed

- Fix memory leak in ArrayList by kngwyu [#316](https://github.com/PyO3/pyo3/pull/316)

## [0.5.2] - 2018-11-25

### Fixed

- Fix undeterministic segfaults when creating many objects by kngwyu in [#281](https://github.com/PyO3/pyo3/pull/281)

## [0.5.1] - 2018-11-24

Yanked

## [0.5.0] - 2018-11-11

### Added

- `#[pyclass]` objects can now be returned from rust functions
- `PyComplex` by kngwyu in [#226](https://github.com/PyO3/pyo3/pull/226)
- `PyDict::from_sequence`, equivalent to `dict([(key, val), ...])`
- Bindings for the `datetime` standard library types: `PyDate`, `PyTime`, `PyDateTime`, `PyTzInfo`, `PyDelta` with associated `ffi` types, by pganssle [#200](https://github.com/PyO3/pyo3/pull/200).
- `PyString`, `PyUnicode`, and `PyBytes` now have an `as_bytes` method that returns `&[u8]`.
- `PyObjectProtocol::get_type_ptr` by ijl in [#242](https://github.com/PyO3/pyo3/pull/242)

### Changed

- Removes the types from the root module and the prelude. They now live in `pyo3::types` instead.
- All exceptions are consturcted with `py_err` instead of `new`, as they return `PyErr` and not `Self`.
- `as_mut` and friends take and `&mut self` instead of `&self`
- `ObjectProtocol::call` now takes an `Option<&PyDict>` for the kwargs instead of an `IntoPyDictPointer`.
- `IntoPyDictPointer` was replace by `IntoPyDict` which doesn't convert `PyDict` itself anymore and returns a `PyDict` instead of `*mut PyObject`.
- `PyTuple::new` now takes an `IntoIterator` instead of a slice
- Updated to syn 0.15
- Splitted `PyTypeObject` into `PyTypeObject` without the create method and `PyTypeCreate` with requires `PyObjectAlloc<Self> + PyTypeInfo + Sized`.
- Ran `cargo edition --fix` which prefixed path with `crate::` for rust 2018
- Renamed `async` to `pyasync` as async will be a keyword in the 2018 edition.
- Starting to use `NonNull<*mut PyObject>` for Py and PyObject by ijl [#260](https://github.com/PyO3/pyo3/pull/260)

### Removed

- Removed most entries from the prelude. The new prelude is small and clear.
- Slowly removing specialization uses
- `PyString`, `PyUnicode`, and `PyBytes` no longer have a `data` method
  (replaced by `as_bytes`) and `PyStringData` has been removed.
- The pyobject_extract macro

### Fixed

- Added an explanation that the GIL can temporarily be released even while holding a GILGuard.
- Lots of clippy errors
- Fix segfault on calling an unknown method on a PyObject
- Work around a [bug](https://github.com/rust-lang/rust/issues/55380) in the rust compiler by kngwyu [#252](https://github.com/PyO3/pyo3/pull/252)
- Fixed a segfault with subclassing pyo3 create classes and using `__class__` by kngwyu [#263](https://github.com/PyO3/pyo3/pull/263)

## [0.4.1] - 2018-08-20

### Changed

- PyTryFrom's error is always to `PyDowncastError`

### Fixed

- Fixed compilation on nightly since `use_extern_macros` was stabilized

### Removed

- The pyobject_downcast macro

## [0.4.0] - 2018-07-30

### Changed

- Merged both examples into one
- Rustfmt all the things :heavy_check_mark:
- Switched to [Keep a Changelog](https://keepachangelog.com/en/1.0.0/)

### Removed

- Conversions from tuples to PyDict due to [rust-lang/rust#52050](https://github.com/rust-lang/rust/issues/52050)

## [0.3.2] - 2018-07-22

### Changed

- Replaced `concat_idents` with mashup

## [0.3.1] - 2018-07-18

### Fixed

- Fixed scoping bug in pyobject_native_type that would break rust-numpy

## [0.3.0] - 2018-07-18

### Added

- A few internal macros became part of the public api ([#155](https://github.com/PyO3/pyo3/pull/155), [#186](https://github.com/PyO3/pyo3/pull/186))
- Always clone in getters. This allows using the get-annotation on all Clone-Types

### Changed

- Upgraded to syn 0.14 which means much better error messages :tada:
- 128 bit integer support by [kngwyu](https://github.com/kngwyu) ([#137](https://github.com/PyO3/pyo3/pull/173))
- `proc_macro` has been stabilized on nightly ([rust-lang/rust#52081](https://github.com/rust-lang/rust/pull/52081)). This means that we can remove the `proc_macro` feature, but now we need the `use_extern_macros` from the 2018 edition instead.
- All proc macro are now prefixed with `py` and live in the prelude. This means you can use `#[pyclass]`, `#[pymethods]`, `#[pyproto]`, `#[pyfunction]` and `#[pymodinit]` directly, at least after a `use pyo3::prelude::*`. They were also moved into a module called `proc_macro`. You shouldn't use `#[pyo3::proc_macro::pyclass]` or other longer paths in attributes because `proc_macro_path_invoc` isn't going to be stabilized soon.
- Renamed the `base` option in the `pyclass` macro to `extends`.
- `#[pymodinit]` uses the function name as module name, unless the name is overrriden with `#[pymodinit(name)]`
- The guide is now properly versioned.

## [0.2.7] - 2018-05-18

### Fixed

- Fix nightly breakage with proc_macro_path

## [0.2.6] - 2018-04-03

### Fixed

- Fix compatibility with TryFrom trait #137

## [0.2.5] - 2018-02-21

### Added

- CPython 3.7 support

### Fixed

- Embedded CPython 3.7b1 crashes on initialization #110
- Generated extension functions are weakly typed #108
- call_method\* crashes when the method does not exist #113
- Allow importing exceptions from nested modules #116

## [0.2.4] - 2018-01-19

### Added

- Allow to get mutable ref from PyObject #106
- Drop `RefFromPyObject` trait
- Add Python::register_any method

### Fixed

- Fix impl `FromPyObject` for `Py<T>`
- Mark method that work with raw pointers as unsafe #95

## [0.2.3] - 11-27-2017

### Changed

- Rustup to 1.23.0-nightly 2017-11-07

### Fixed

- Proper `c_char` usage #93

### Removed

- Remove use of now unneeded 'AsciiExt' trait

## [0.2.2] - 09-26-2017

### Changed

- Rustup to 1.22.0-nightly 2017-09-30

## [0.2.1] - 09-26-2017

### Fixed

- Fix rustc const_fn nightly breakage

## [0.2.0] - 08-12-2017

### Added

- Added inheritance support #15
- Added weakref support #56
- Added subclass support #64
- Added `self.__dict__` supoort #68
- Added `pyo3::prelude` module #70
- Better `Iterator` support for PyTuple, PyList, PyDict #75
- Introduce IntoPyDictPointer similar to IntoPyTuple #69

### Changed

- Allow to add gc support without implementing PyGCProtocol #57
- Refactor `PyErr` implementation. Drop `py` parameter from constructor.

## [0.1.0] - 07-23-2017

### Added

- Initial release

[Unreleased]: https://github.com/pyo3/pyo3/compare/v0.16.4...HEAD
[0.16.3]: https://github.com/pyo3/pyo3/compare/v0.16.3...v0.16.4
[0.16.3]: https://github.com/pyo3/pyo3/compare/v0.16.2...v0.16.3
[0.16.2]: https://github.com/pyo3/pyo3/compare/v0.16.1...v0.16.2
[0.16.1]: https://github.com/pyo3/pyo3/compare/v0.16.0...v0.16.1
[0.16.0]: https://github.com/pyo3/pyo3/compare/v0.15.1...v0.16.0
[0.15.2]: https://github.com/pyo3/pyo3/compare/v0.15.1...v0.15.2
[0.15.1]: https://github.com/pyo3/pyo3/compare/v0.15.0...v0.15.1
[0.15.0]: https://github.com/pyo3/pyo3/compare/v0.14.5...v0.15.0
[0.14.5]: https://github.com/pyo3/pyo3/compare/v0.14.4...v0.14.5
[0.14.4]: https://github.com/pyo3/pyo3/compare/v0.14.3...v0.14.4
[0.14.3]: https://github.com/pyo3/pyo3/compare/v0.14.2...v0.14.3
[0.14.2]: https://github.com/pyo3/pyo3/compare/v0.14.1...v0.14.2
[0.14.1]: https://github.com/pyo3/pyo3/compare/v0.14.0...v0.14.1
[0.14.0]: https://github.com/pyo3/pyo3/compare/v0.13.2...v0.14.0
[0.13.2]: https://github.com/pyo3/pyo3/compare/v0.13.1...v0.13.2
[0.13.1]: https://github.com/pyo3/pyo3/compare/v0.13.0...v0.13.1
[0.13.0]: https://github.com/pyo3/pyo3/compare/v0.12.4...v0.13.0
[0.12.4]: https://github.com/pyo3/pyo3/compare/v0.12.3...v0.12.4
[0.12.3]: https://github.com/pyo3/pyo3/compare/v0.12.2...v0.12.3
[0.12.2]: https://github.com/pyo3/pyo3/compare/v0.12.1...v0.12.2
[0.12.1]: https://github.com/pyo3/pyo3/compare/v0.12.0...v0.12.1
[0.12.0]: https://github.com/pyo3/pyo3/compare/v0.11.1...v0.12.0
[0.11.1]: https://github.com/pyo3/pyo3/compare/v0.11.0...v0.11.1
[0.11.0]: https://github.com/pyo3/pyo3/compare/v0.10.1...v0.11.0
[0.10.1]: https://github.com/pyo3/pyo3/compare/v0.10.0...v0.10.1
[0.10.0]: https://github.com/pyo3/pyo3/compare/v0.9.2...v0.10.0
[0.9.2]: https://github.com/pyo3/pyo3/compare/v0.9.1...v0.9.2
[0.9.1]: https://github.com/pyo3/pyo3/compare/v0.9.0...v0.9.1
[0.9.0]: https://github.com/pyo3/pyo3/compare/v0.8.5...v0.9.0
[0.8.4]: https://github.com/pyo3/pyo3/compare/v0.8.4...v0.8.5
[0.8.4]: https://github.com/pyo3/pyo3/compare/v0.8.3...v0.8.4
[0.8.3]: https://github.com/pyo3/pyo3/compare/v0.8.2...v0.8.3
[0.8.2]: https://github.com/pyo3/pyo3/compare/v0.8.1...v0.8.2
[0.8.1]: https://github.com/pyo3/pyo3/compare/v0.8.0...v0.8.1
[0.8.0]: https://github.com/pyo3/pyo3/compare/v0.7.0...v0.8.0
[0.7.0]: https://github.com/pyo3/pyo3/compare/v0.6.0...v0.7.0
[0.6.0]: https://github.com/pyo3/pyo3/compare/v0.5.3...v0.6.0
[0.5.3]: https://github.com/pyo3/pyo3/compare/v0.5.2...v0.5.3
[0.5.2]: https://github.com/pyo3/pyo3/compare/v0.5.0...v0.5.2
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
[0.1.0]: https://github.com/PyO3/pyo3/tree/0.1.0
