# Changelog
All notable changes to this project will be documented in this file. For help with updating to new
PyO3 versions, please see the [migration guide](https://pyo3.rs/master/migration.html).

The format is based on [Keep a Changelog](http://keepachangelog.com/en/1.0.0/)
and this project adheres to [Semantic Versioning](http://semver.org/spec/v2.0.0.html).

## [0.13.2] - 2021-02-12
### Packaging
- Lower minimum supported Rust version to 1.41. [#1421](https://github.com/PyO3/pyo3/pull/1421)

### Added
- Add unsafe API `with_embedded_python_interpreter` to  initalize a Python interpreter, execute a closure, and finalize the interpreter. [#1355](https://github.com/PyO3/pyo3/pull/1355)
- Add `serde` feature which provides implementations of `Serialize` and `Deserialize` for `Py<T>`. [#1366](https://github.com/PyO3/pyo3/pull/1366)
- Add FFI definition `_PyCFunctionFastWithKeywords` on Python 3.7 and up. [#1384](https://github.com/PyO3/pyo3/pull/1384)
- Add `PyDateTime::new_with_fold()` method. [#1398](https://github.com/PyO3/pyo3/pull/1398)

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
- Add `PyAny::is_instance()` method. [#1276](https://github.com/PyO3/pyo3/pull/1276)
- Add support for conversion between `char` and `PyString`. [#1282](https://github.com/PyO3/pyo3/pull/1282)
- Add FFI definitions for `PyBuffer_SizeFromFormat`, `PyObject_LengthHint`, `PyObject_CallNoArgs`, `PyObject_CallOneArg`, `PyObject_CallMethodNoArgs`, `PyObject_CallMethodOneArg`, `PyObject_VectorcallDict`, and `PyObject_VectorcallMethod`. [#1287](https://github.com/PyO3/pyo3/pull/1287)
- Add conversions between `u128`/`i128` and `PyLong` for PyPy. [#1310](https://github.com/PyO3/pyo3/pull/1310)
- Add `Python::version()` and `Python::version_info()` to get the running interpreter version. [#1322](https://github.com/PyO3/pyo3/pull/1322)

### Changed
- Change return type of `PyType::name()` from `Cow<str>` to `PyResult<&str>`. [#1152](https://github.com/PyO3/pyo3/pull/1152)
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
- Add `Python::check_signals()` as a safe a wrapper for `PyErr_CheckSignals()`. [#1214](https://github.com/PyO3/pyo3/pull/1214)

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
- Add type information to failures in `PyAny::downcast()`. [#1050](https://github.com/PyO3/pyo3/pull/1050)
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
  - Rename `PyException::py_err()` to `PyException::new_err()`.
  - Rename `PyUnicodeDecodeErr::new_err()` to `PyUnicodeDecodeErr::new()`.
  - Remove `PyStopIteration::stop_iteration()`.
- Require `T: Send` for the return value `T` of `Python::allow_threads`. [#1036](https://github.com/PyO3/pyo3/pull/1036)
- Rename `PYTHON_SYS_EXECUTABLE` to `PYO3_PYTHON`. The old name will continue to work (undocumented) but will be removed in a future release. [#1039](https://github.com/PyO3/pyo3/pull/1039)
- Remove `unsafe` from signature of `PyType::as_type_ptr`. [#1047](https://github.com/PyO3/pyo3/pull/1047)
- Change return type of `PyIterator::from_object` to `PyResult<PyIterator>` (was `Result<PyIterator, PyDowncastError>`). [#1051](https://github.com/PyO3/pyo3/pull/1051)
- `IntoPy` is no longer implied by `FromPy`. [#1063](https://github.com/PyO3/pyo3/pull/1063)
- Change `PyObject` to be a type alias for `Py<PyAny>`. [#1063](https://github.com/PyO3/pyo3/pull/1063)
- Rework `PyErr` to be compatible with the `std::error::Error` trait: [#1067](https://github.com/PyO3/pyo3/pull/1067) [#1115](https://github.com/PyO3/pyo3/pull/1115)
  - Implement `Display`, `Error`, `Send` and `Sync` for `PyErr` and `PyErrArguments`.
  - Add `PyErr::instance()` for accessing `PyErr` as `&PyBaseException`.
  - `PyErr`'s fields are now an implementation detail. The equivalent values can be accessed with `PyErr::ptype()`, `PyErr::pvalue()` and `PyErr::ptraceback()`.
  - Change receiver of `PyErr::print()` and `PyErr::print_and_set_sys_last_vars()` to `&self` (was `self`).
  - Remove `PyErrValue`, `PyErr::from_value`, `PyErr::into_normalized()`, and `PyErr::normalize()`.
  - Remove `PyException::into()`.
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
- Fix the `+` operator not trying `__radd__` when both `__add__` and `__radd__`  are defined in `PyNumberProtocol` (and similar for all other reversible operators). [#1107](https://github.com/PyO3/pyo3/pull/1107)
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
- Change signature of `PyTypeObject::type_object()` - now takes `Python` argument and returns `&PyType`. [#970](https://github.com/PyO3/pyo3/pull/970)
- Change return type of `PyTuple::slice()` and `PyTuple::split_from()` from `Py<PyTuple>` to `&PyTuple`. [#970](https://github.com/PyO3/pyo3/pull/970)
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
- Fix deadlock in `Python::acquire_gil()` after dropping a `PyObject` or `Py<T>`. [#924](https://github.com/PyO3/pyo3/pull/924)

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
- The blanket implementations for `FromPyObject` for `&T` and `&mut T` are no longer specializable. Implement `PyTryFrom` for your type to control the behavior of `FromPyObject::extract()` for your types. [#713](https://github.com/PyO3/pyo3/pull/713)
- The implementation for `IntoPy<U> for T` where `U: FromPy<T>` is no longer specializable. Control the behavior of this via the implementation of `FromPy`. [#713](https://github.com/PyO3/pyo3/pull/713)
- Use `parking_lot::Mutex` instead of `spin::Mutex`. [#734](https://github.com/PyO3/pyo3/pull/734)
- Bumped minimum Rust version to `1.42.0-nightly 2020-01-21`. [#761](https://github.com/PyO3/pyo3/pull/761)
- `PyRef` and `PyRefMut` are renewed for `PyCell`. [#770](https://github.com/PyO3/pyo3/pull/770)
- Some new FFI functions for Python 3.8. [#784](https://github.com/PyO3/pyo3/pull/784)
- `PyAny` is now on the top level module and prelude. [#816](https://github.com/PyO3/pyo3/pull/816)

### Removed
- `PyRawObject`. [#683](https://github.com/PyO3/pyo3/pull/683)
- `PyNoArgsFunction`. [#741](https://github.com/PyO3/pyo3/pull/741)
- `initialize_type()`. To set the module name for a `#[pyclass]`, use the `module` argument to the macro. #[751](https://github.com/PyO3/pyo3/pull/751)
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
- Use existing fields and methods before calling custom __getattr__. [#505](https://github.com/PyO3/pyo3/pull/505)
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
- `PyDict::from_sequence()`, equivalent to `dict([(key, val), ...])`
- Bindings for the `datetime` standard library types: `PyDate`, `PyTime`, `PyDateTime`, `PyTzInfo`, `PyDelta` with associated `ffi` types, by pganssle [#200](https://github.com/PyO3/pyo3/pull/200).
- `PyString`, `PyUnicode`, and `PyBytes` now have an `as_bytes()` method that returns `&[u8]`.
- `PyObjectProtocol::get_type_ptr()` by ijl in [#242](https://github.com/PyO3/pyo3/pull/242)

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
- `PyString`, `PyUnicode`, and `PyBytes` no longer have a `data()` method
 (replaced by `as_bytes()`) and `PyStringData` has been removed.
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
- Switched to [Keep a Changelog](http://keepachangelog.com/en/1.0.0/)

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
- call_method*() crashes when the method does not exist #113
- Allow importing exceptions from nested modules #116

## [0.2.4] - 2018-01-19
### Added
- Allow to get mutable ref from PyObject #106
- Drop `RefFromPyObject` trait
- Add Python::register_any() method

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

[Unreleased]: https://github.com/pyo3/pyo3/compare/v0.13.2...HEAD
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
