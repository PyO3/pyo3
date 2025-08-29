# Changelog

All notable changes to this project will be documented in this file. For help with updating to new
PyO3 versions, please see the [migration guide](https://pyo3.rs/latest/migration.html).

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/)
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

To see unreleased changes, please see the [CHANGELOG on the main branch guide](https://pyo3.rs/main/changelog.html).

<!-- towncrier release notes start -->

## [0.26.0] - 2025-08-29

### Packaging

- Bump hashbrown dependency to 0.15. [#5152](https://github.com/PyO3/pyo3/pull/5152)
- Update MSRV to 1.74. [#5171](https://github.com/PyO3/pyo3/pull/5171)
- Set the same maximum supported version for alternative interpreters as for CPython. [#5192](https://github.com/PyO3/pyo3/pull/5192)
- Add optional `bytes` dependency to add conversions for `bytes::Bytes`. [#5252](https://github.com/PyO3/pyo3/pull/5252)
- Publish new crate `pyo3-introspection` to pair with the `experimental-inspect` feature. [#5300](https://github.com/PyO3/pyo3/pull/5300)
- The `PYO3_BUILD_EXTENSION_MODULE` now causes the same effect as the `extension-module` feature. Eventually we expect maturin and setuptools-rust to set this environment variable automatically. Users with their own build systems will need to do the same. [#5343](https://github.com/PyO3/pyo3/pull/5343)

### Added

- Add `#[pyo3(warn(message = "...", category = ...))]` attribute for automatic warnings generation for `#[pyfunction]` and `#[pymethods]`. [#4364](https://github.com/PyO3/pyo3/pull/4364)
- Add `PyMutex`, available on Python 3.13 and newer. [#4523](https://github.com/PyO3/pyo3/pull/4523)
- Add FFI definition `PyMutex_IsLocked`, available on Python 3.14 and newer. [#4523](https://github.com/PyO3/pyo3/pull/4523)
- Add `PyString::from_encoded_object`. [#5017](https://github.com/PyO3/pyo3/pull/5017)
- `experimental-inspect`: add basic input type annotations. [#5089](https://github.com/PyO3/pyo3/pull/5089)
- Add FFI function definitions for `PyFrameObject` from CPython 3.13. [#5154](https://github.com/PyO3/pyo3/pull/5154)
- `experimental-inspect`: tag modules created using `#[pymodule]` or `#[pymodule_init]` functions as incomplete. [#5207](https://github.com/PyO3/pyo3/pull/5207)
- `experimental-inspect`: add basic return type support. [#5208](https://github.com/PyO3/pyo3/pull/5208)
- Add `PyCode::compile` and `PyCodeMethods::run` to create and execute code objects. [#5217](https://github.com/PyO3/pyo3/pull/5217)
- Add `PyOnceLock` type for thread-safe single-initialization. [#5223](https://github.com/PyO3/pyo3/pull/5223)
- Add `PyClassGuard(Mut)` pyclass holders. In the future they will replace `PyRef(Mut)`. [#5233](https://github.com/PyO3/pyo3/pull/5233)
- `experimental-inspect`: allow annotations in `#[pyo3(signature)]` signature attribute. [#5241](https://github.com/PyO3/pyo3/pull/5241)
- Implement `MutexExt` for parking_lot's/lock_api `ReentrantMutex`. [#5258](https://github.com/PyO3/pyo3/pull/5258)
- `experimental-inspect`: support class associated constants. [#5272](https://github.com/PyO3/pyo3/pull/5272)
- Add `Bound::cast` family of functions superseding the `PyAnyMethods::downcast` family. [#5289](https://github.com/PyO3/pyo3/pull/5289)
- Add FFI definitions `Py_Version` and `Py_IsFinalizing`. [#5317](https://github.com/PyO3/pyo3/pull/5317)
- `experimental-inspect`: add output type annotation for `#[pyclass]`. [#5320](https://github.com/PyO3/pyo3/pull/5320)
- `experimental-inspect`: support `#[pyclass(eq, eq_int, ord, hash, str)]`. [#5338](https://github.com/PyO3/pyo3/pull/5338)
- `experimental-inspect`: add basic support for `#[derive(FromPyObject)]` (no struct fields support yet). [#5339](https://github.com/PyO3/pyo3/pull/5339)
- Add `Python::try_attach`. [#5342](https://github.com/PyO3/pyo3/pull/5342)

### Changed

- Use `Py_TPFLAGS_DISALLOW_INSTANTIATION` instead of a `__new__` which always fails for a `#[pyclass]` without a `#[new]` on Python 3.10 and up. [#4568](https://github.com/PyO3/pyo3/pull/4568)
- `PyModule::from_code` now defaults `file_name` to `<string>` if empty. [#4777](https://github.com/PyO3/pyo3/pull/4777)
- Deprecate `PyString::from_object` in favour of `PyString::from_encoded_object`. [#5017](https://github.com/PyO3/pyo3/pull/5017)
- When building with `abi3` for a Python version newer than pyo3 supports, automatically fall back to an abi3 build for the latest supported version. [#5144](https://github.com/PyO3/pyo3/pull/5144)
- Change `is_instance_of` trait bound from `PyTypeInfo` to `PyTypeCheck`. [#5146](https://github.com/PyO3/pyo3/pull/5146)
- Many PyO3 proc macros now report multiple errors instead of only the first one. [#5159](https://github.com/PyO3/pyo3/pull/5159)
- Change `MutexExt` return type to be an associated type. [#5201](https://github.com/PyO3/pyo3/pull/5201)
- Use `PyCallArgs` for `Py::call` and friends so they're equivalent to their `Bound` counterpart. [#5206](https://github.com/PyO3/pyo3/pull/5206)
- Rename `Python::with_gil` to `Python::attach`. [#5209](https://github.com/PyO3/pyo3/pull/5209)
- Rename `Python::allow_threads` to `Python::detach` [#5221](https://github.com/PyO3/pyo3/pull/5221)
- Deprecate `GILOnceCell` type in favour of `PyOnceLock`. [#5223](https://github.com/PyO3/pyo3/pull/5223)
- Rename `pyo3::prepare_freethreaded_python` to `Python::initialize`. [#5247](https://github.com/PyO3/pyo3/pull/5247)
- Convert `PyMemoryError` into/from `io::ErrorKind::OutOfMemory`. [#5256](https://github.com/PyO3/pyo3/pull/5256)
- Deprecate `GILProtected`. [#5285](https://github.com/PyO3/pyo3/pull/5285)
- Move `#[pyclass]` docstring formatting from import time to compile time. [#5286](https://github.com/PyO3/pyo3/pull/5286)
- `Python::attach` will now panic if the Python interpreter is in the process of shutting down. [#5317](https://github.com/PyO3/pyo3/pull/5317)
- Add fast-path to `PyTypeInfo::type_object` for `#[pyclass]` types. [#5324](https://github.com/PyO3/pyo3/pull/5324)
- Deprecate `PyObject` type alias for `Py<PyAny>`. [#5325](https://github.com/PyO3/pyo3/pull/5325)
- Rename `Python::with_gil_unchecked` to `Python::attach_unchecked`. [#5340](https://github.com/PyO3/pyo3/pull/5340)
- Rename `Python::assume_gil_acquired` to `Python::assume_attached`. [#5354](https://github.com/PyO3/pyo3/pull/5354)

### Removed

- Remove FFI definition of internals of `PyFrameObject`. [#5154](https://github.com/PyO3/pyo3/pull/5154)
- Remove `Eq` and `PartialEq` implementations on `PyGetSetDef` FFI definition. [#5196](https://github.com/PyO3/pyo3/pull/5196)
- Remove private FFI definitions `_Py_IsCoreInitialized` and `_Py_InitializeMain`. [#5317](https://github.com/PyO3/pyo3/pull/5317)

### Fixed

- Use critical section in `PyByteArray::to_vec` on freethreaded build to replicate GIL-enabled "soundness". [#4742](https://github.com/PyO3/pyo3/pull/4742)
- Fix precision loss when converting `bigdecimal` into Python. [#5198](https://github.com/PyO3/pyo3/pull/5198)
- Don't treat win7 target as a cross-compilation. [#5210](https://github.com/PyO3/pyo3/pull/5210)
- WASM targets no longer require exception handling support for Python < 3.14. [#5239](https://github.com/PyO3/pyo3/pull/5239)
- Fix segfault when dropping `PyBuffer<T>` after the Python interpreter has been finalized. [#5242](https://github.com/PyO3/pyo3/pull/5242)
- `experimental-inspect`: better automated imports generation. [#5251](https://github.com/PyO3/pyo3/pull/5251)
- `experimental-inspect`: fix introspection of `__richcmp__`, `__concat__`, `__repeat__`, `__inplace_concat__` and `__inplace_repeat__`. [#5273](https://github.com/PyO3/pyo3/pull/5273)
- fixed a leaked borrow, when converting a mutable sub class into a frozen base class using `PyRef::into_super` [#5281](https://github.com/PyO3/pyo3/pull/5281)
- Fix FFI definition `Py_Exit` (never returns, was `()` return value, now `!`). [#5317](https://github.com/PyO3/pyo3/pull/5317)
- `experimental-inspect`: fix handling of module members gated behind `#[cfg(...)]` attributes. [#5318](https://github.com/PyO3/pyo3/pull/5318)

## [0.25.1] - 2025-06-12
### Packaging

- Add support for Windows on ARM64. [#5145](https://github.com/PyO3/pyo3/pull/5145)
- Add `chrono-local` feature for optional conversions for chrono's `Local` timezone & `DateTime<Local>` instances. [#5174](https://github.com/PyO3/pyo3/pull/5174)

### Added

- Add FFI definition `PyBytes_AS_STRING`. [#5121](https://github.com/PyO3/pyo3/pull/5121)
- Add support for module associated consts introspection. [#5150](https://github.com/PyO3/pyo3/pull/5150)

### Changed

- Enable "vectorcall" FFI definitions on GraalPy. [#5121](https://github.com/PyO3/pyo3/pull/5121)
- Use `Py_Is` function on GraalPy [#5121](https://github.com/PyO3/pyo3/pull/5121)

### Fixed

- Report a better compile error for `async` declarations when not using `experimental-async` feature. [#5156](https://github.com/PyO3/pyo3/pull/5156)
- Fix implementation of `FromPyObject` for `uuid::Uuid` on big-endian architectures. [#5161](https://github.com/PyO3/pyo3/pull/5161)
- Fix segmentation faults on 32-bit x86 with Python 3.14. [#5180](https://github.com/PyO3/pyo3/pull/5180)

## [0.25.0] - 2025-05-14

### Packaging

- Support Python 3.14.0b1. [#4811](https://github.com/PyO3/pyo3/pull/4811)
- Bump supported GraalPy version to 24.2. [#5116](https://github.com/PyO3/pyo3/pull/5116)
- Add optional `bigdecimal` dependency to add conversions for `bigdecimal::BigDecimal`. [#5011](https://github.com/PyO3/pyo3/pull/5011)
- Add optional `time` dependency to add conversions for `time` types. [#5057](https://github.com/PyO3/pyo3/pull/5057)
- Remove `cfg-if` dependency. [#5110](https://github.com/PyO3/pyo3/pull/5110)
- Add optional `ordered_float` dependency to add conversions for `ordered_float::NotNan` and `ordered_float::OrderedFloat`. [#5114](https://github.com/PyO3/pyo3/pull/5114)

### Added

- Add initial type stub generation to the `experimental-inspect` feature. [#3977](https://github.com/PyO3/pyo3/pull/3977)
- Add `#[pyclass(generic)]` option to support runtime generic typing. [#4926](https://github.com/PyO3/pyo3/pull/4926)
- Implement `OnceExt` & `MutexExt` for `parking_lot` & `lock_api`. Use the new extension traits by enabling the `arc_lock`, `lock_api`, or `parking_lot` cargo features. [#5044](https://github.com/PyO3/pyo3/pull/5044)
- Implement `From`/`Into` for `Borrowed<T>` -> `Py<T>`. [#5054](https://github.com/PyO3/pyo3/pull/5054)
- Add `PyTzInfo` constructors. [#5055](https://github.com/PyO3/pyo3/pull/5055)
- Add FFI definition `PY_INVALID_STACK_EFFECT`. [#5064](https://github.com/PyO3/pyo3/pull/5064)
- Implement `AsRef<Py<PyAny>>` for `Py<T>`, `Bound<T>` and `Borrowed<T>`. [#5071](https://github.com/PyO3/pyo3/pull/5071)
- Add FFI definition `PyModule_Add` and `compat::PyModule_Add`. [#5085](https://github.com/PyO3/pyo3/pull/5085)
- Add FFI definitions `Py_HashBuffer`, `Py_HashPointer`, and `PyObject_GenericHash`. [#5086](https://github.com/PyO3/pyo3/pull/5086)
- Support `#[pymodule_export]` on `const` items in declarative modules. [#5096](https://github.com/PyO3/pyo3/pull/5096)
- Add `#[pyclass(immutable_type)]` option (on Python 3.14+ with `abi3`, or 3.10+ otherwise) for immutable type objects. [#5101](https://github.com/PyO3/pyo3/pull/5101)
- Support `#[pyo3(rename_all)]` support on `#[derive(IntoPyObject)]`. [#5112](https://github.com/PyO3/pyo3/pull/5112)
- Add `PyRange` wrapper. [#5117](https://github.com/PyO3/pyo3/pull/5117)

### Changed

- Enable use of `datetime` types with `abi3` feature enabled. [#4970](https://github.com/PyO3/pyo3/pull/4970)
- Deprecate `timezone_utc` in favor of `PyTzInfo::utc`. [#5055](https://github.com/PyO3/pyo3/pull/5055)
- Reduce visibility of some CPython implementation details: [#5064](https://github.com/PyO3/pyo3/pull/5064)
  - The FFI definition `PyCodeObject` is now an opaque struct on all Python versions.
  - The FFI definition `PyFutureFeatures` is now only defined up until Python 3.10 (it was present in CPython headers but unused in 3.11 and 3.12).
- Change `PyAnyMethods::is` to take `other: &Bound<T>`. [#5071](https://github.com/PyO3/pyo3/pull/5071)
- Change `Py::is` to take `other: &Py<T>`. [#5071](https://github.com/PyO3/pyo3/pull/5071)
- Change `PyVisit::call` to take `T: Into<Option<&Py<T>>>`. [#5071](https://github.com/PyO3/pyo3/pull/5071)
- Expose `PyDateTime_DATE_GET_TZINFO` and `PyDateTime_TIME_GET_TZINFO` on PyPy 3.10 and later. [#5079](https://github.com/PyO3/pyo3/pull/5079)
- Add `#[track_caller]` to `with_gil` and `with_gil_unchecked`. [#5109](https://github.com/PyO3/pyo3/pull/5109)
- Use `std::thread::park()` instead of `libc::pause()` or `sleep(9999999)`. [#5115](https://github.com/PyO3/pyo3/pull/5115)

### Removed

- Remove all functionality deprecated in PyO3 0.23. [#4982](https://github.com/PyO3/pyo3/pull/4982)
- Remove deprecated `IntoPy` and `ToPyObject` traits. [#5010](https://github.com/PyO3/pyo3/pull/5010)
- Remove private types from `pyo3-ffi` (i.e. starting with `_Py`) which are not referenced by public APIs: `_PyLocalMonitors`, `_Py_GlobalMonitors`, `_PyCoCached`, `_PyCoLineInstrumentationData`, `_PyCoMonitoringData`, `_PyCompilerSrcLocation`, `_PyErr_StackItem`. [#5064](https://github.com/PyO3/pyo3/pull/5064)
- Remove FFI definition `PyCode_GetNumFree` (PyO3 cannot support it due to knowledge of the code object). [#5064](https://github.com/PyO3/pyo3/pull/5064)
- Remove `AsPyPointer` trait. [#5071](https://github.com/PyO3/pyo3/pull/5071)
- Remove support for the deprecated string form of `from_py_with`. [#5097](https://github.com/PyO3/pyo3/pull/5097)
- Remove FFI definitions of private static variables: `_PyMethodWrapper_Type`, `_PyCoroWrapper_Type`, `_PyImport_FrozenBootstrap`, `_PyImport_FrozenStdlib`, `_PyImport_FrozenTest`, `_PyManagedBuffer_Type`, `_PySet_Dummy`, `_PyWeakref_ProxyType`, and `_PyWeakref_CallableProxyType`. [#5105](https://github.com/PyO3/pyo3/pull/5105)
- Remove FFI definitions `PyASCIIObjectState`, `PyUnicode_IS_ASCII`, `PyUnicode_IS_COMPACT`, and `PyUnicode_IS_COMPACT_ASCII` on Python 3.14 and newer. [#5133](https://github.com/PyO3/pyo3/pull/5133)

### Fixed

- Correctly pick up the shared state for conda-based Python installation when reading information from sysconfigdata. [#5037](https://github.com/PyO3/pyo3/pull/5037)
- Fix compile failure with `#[derive(IntoPyObject, FromPyObject)]` when using `#[pyo3()]` options recognised by only one of the two derives. [#5070](https://github.com/PyO3/pyo3/pull/5070)
- Fix various compile errors from missing FFI definitions using certain feature combinations on PyPy and GraalPy. [#5091](https://github.com/PyO3/pyo3/pull/5091)
- Fallback on `backports.zoneinfo` for python <3.9 when converting timezones into python. [#5120](https://github.com/PyO3/pyo3/pull/5120)

## [0.24.2] - 2025-04-21

### Fixed

- Fix `unused_imports` lint of `#[pyfunction]` and `#[pymethods]` expanded in `macro_rules` context. [#5030](https://github.com/PyO3/pyo3/pull/5030)
- Fix size of `PyCodeObject::_co_instrumentation_version` ffi struct member on Python 3.13 for systems where `uintptr_t` is not 64 bits. [#5048](https://github.com/PyO3/pyo3/pull/5048)
- Fix struct-type complex enum variant fields incorrectly exposing raw identifiers as `r#ident` in Python bindings. [#5050](https://github.com/PyO3/pyo3/pull/5050)

## [0.24.1] - 2025-03-31

### Added

- Add `abi3-py313` feature. [#4969](https://github.com/PyO3/pyo3/pull/4969)
- Add `PyAnyMethods::getattr_opt`. [#4978](https://github.com/PyO3/pyo3/pull/4978)
- Add `PyInt::new` constructor for all supported number types (i32, u32, i64, u64, isize, usize). [#4984](https://github.com/PyO3/pyo3/pull/4984)
- Add `pyo3::sync::with_critical_section2`. [#4992](https://github.com/PyO3/pyo3/pull/4992)
- Implement `PyCallArgs` for `Borrowed<'_, 'py, PyTuple>`, `&Bound<'py, PyTuple>`, and `&Py<PyTuple>`. [#5013](https://github.com/PyO3/pyo3/pull/5013)

### Fixed

- Fix `is_type_of` for native types not using same specialized check as `is_type_of_bound`. [#4981](https://github.com/PyO3/pyo3/pull/4981)
- Fix `Probe` class naming issue with `#[pymethods]`. [#4988](https://github.com/PyO3/pyo3/pull/4988)
- Fix compile failure with required `#[pyfunction]` arguments taking `Option<&str>` and `Option<&T>` (for `#[pyclass]` types). [#5002](https://github.com/PyO3/pyo3/pull/5002)
- Fix `PyString::from_object` causing of bounds reads whith `encoding` and `errors` parameters which are not nul-terminated. [#5008](https://github.com/PyO3/pyo3/pull/5008)
- Fix compile error when additional options follow after `crate` for `#[pyfunction]`. [#5015](https://github.com/PyO3/pyo3/pull/5015)

## [0.24.0] - 2025-03-09

### Packaging

- Add supported CPython/PyPy versions to cargo package metadata. [#4756](https://github.com/PyO3/pyo3/pull/4756)
- Bump `target-lexicon` dependency to 0.13. [#4822](https://github.com/PyO3/pyo3/pull/4822)
- Add optional `jiff` dependency to add conversions for `jiff` datetime types. [#4823](https://github.com/PyO3/pyo3/pull/4823)
- Add optional `uuid` dependency to add conversions for `uuid::Uuid`. [#4864](https://github.com/PyO3/pyo3/pull/4864)
- Bump minimum supported `inventory` version to 0.3.5. [#4954](https://github.com/PyO3/pyo3/pull/4954)

### Added

- Add `PyIterator::send` method to allow sending values into a python generator. [#4746](https://github.com/PyO3/pyo3/pull/4746)
- Add `PyCallArgs` trait for passing arguments into the Python calling protocol. This enabled using a faster calling convention for certain types, improving performance. [#4768](https://github.com/PyO3/pyo3/pull/4768)
- Add `#[pyo3(default = ...']` option for `#[derive(FromPyObject)]` to set a default value for extracted fields of named structs. [#4829](https://github.com/PyO3/pyo3/pull/4829)
- Add `#[pyo3(into_py_with = ...)]` option for `#[derive(IntoPyObject, IntoPyObjectRef)]`. [#4850](https://github.com/PyO3/pyo3/pull/4850)
- Add FFI definitions `PyThreadState_GetFrame` and `PyFrame_GetBack`. [#4866](https://github.com/PyO3/pyo3/pull/4866)
- Optimize `last` for `BoundListIterator`, `BoundTupleIterator` and `BorrowedTupleIterator`. [#4878](https://github.com/PyO3/pyo3/pull/4878)
- Optimize `Iterator::count()` for `PyDict`, `PyList`, `PyTuple` & `PySet`. [#4878](https://github.com/PyO3/pyo3/pull/4878)
- Optimize `nth`, `nth_back`, `advance_by` and `advance_back_by` for `BoundTupleIterator` [#4897](https://github.com/PyO3/pyo3/pull/4897)
- Add support for `types.GenericAlias` as `pyo3::types::PyGenericAlias`. [#4917](https://github.com/PyO3/pyo3/pull/4917)
- Add `MutextExt` trait to help avoid deadlocks with the GIL while locking a `std::sync::Mutex`. [#4934](https://github.com/PyO3/pyo3/pull/4934)
- Add `#[pyo3(rename_all = "...")]` option for `#[derive(FromPyObject)]`. [#4941](https://github.com/PyO3/pyo3/pull/4941)

### Changed

- Optimize `nth`, `nth_back`, `advance_by` and `advance_back_by` for `BoundListIterator`. [#4810](https://github.com/PyO3/pyo3/pull/4810)
- Use `DerefToPyAny` in blanket implementations of `From<Py<T>>` and `From<Bound<'py, T>>` for `PyObject`. [#4593](https://github.com/PyO3/pyo3/pull/4593)
- Map `io::ErrorKind::IsADirectory`/`NotADirectory` to the corresponding Python exception on Rust 1.83+. [#4747](https://github.com/PyO3/pyo3/pull/4747)
- `PyAnyMethods::call` and friends now require `PyCallArgs` for their positional arguments. [#4768](https://github.com/PyO3/pyo3/pull/4768)
- Expose FFI definitions for `PyObject_Vectorcall(Method)` on the stable abi on 3.12+. [#4853](https://github.com/PyO3/pyo3/pull/4853)
- `#[pyo3(from_py_with = ...)]` now take a path rather than a string literal [#4860](https://github.com/PyO3/pyo3/pull/4860)
- Format Python traceback in impl Debug for PyErr. [#4900](https://github.com/PyO3/pyo3/pull/4900)
- Convert `PathBuf` & `Path` into Python `pathlib.Path` instead of `PyString`. [#4925](https://github.com/PyO3/pyo3/pull/4925)
- Relax parsing of exotic Python versions. [#4949](https://github.com/PyO3/pyo3/pull/4949)
- PyO3 threads now hang instead of `pthread_exit` trying to acquire the GIL when the interpreter is shutting down. This mimics the [Python 3.14](https://github.com/python/cpython/issues/87135) behavior and avoids undefined behavior and crashes. [#4874](https://github.com/PyO3/pyo3/pull/4874)

### Removed

- Remove implementations of `Deref` for `PyAny` and other "native" types. [#4593](https://github.com/PyO3/pyo3/pull/4593)
- Remove implicit default of trailing optional arguments (see #2935) [#4729](https://github.com/PyO3/pyo3/pull/4729)
- Remove the deprecated implicit eq fallback for simple enums. [#4730](https://github.com/PyO3/pyo3/pull/4730)

### Fixed

- Correct FFI definition of `PyIter_Send` to return a `PySendResult`. [#4746](https://github.com/PyO3/pyo3/pull/4746)
- Fix a thread safety issue in the runtime borrow checker used by mutable pyclass instances on the free-threaded build. [#4948](https://github.com/PyO3/pyo3/pull/4948)


## [0.23.5] - 2025-02-22

### Packaging

- Add support for PyPy3.11 [#4760](https://github.com/PyO3/pyo3/pull/4760)

### Fixed

- Fix thread-unsafe implementation of freelist pyclasses on the free-threaded build. [#4902](https://github.com/PyO3/pyo3/pull/4902)
- Re-enable a workaround for situations where CPython incorrectly does not add `__builtins__` to `__globals__` in code executed by `Python::py_run` (was removed in PyO3 0.23.0). [#4921](https://github.com/PyO3/pyo3/pull/4921)

## [0.23.4] - 2025-01-10

### Added

- Add `PyList::locked_for_each`, which uses a critical section to lock the list on the free-threaded build. [#4789](https://github.com/PyO3/pyo3/pull/4789)
- Add `pyo3_build_config::add_python_framework_link_args` build script API to set rpath when using macOS system Python. [#4833](https://github.com/PyO3/pyo3/pull/4833)

### Changed

- Use `datetime.fold` to distinguish ambiguous datetimes when converting to and from `chrono::DateTime<Tz>` (rather than erroring). [#4791](https://github.com/PyO3/pyo3/pull/4791)
- Optimize PyList iteration on the free-threaded build. [#4789](https://github.com/PyO3/pyo3/pull/4789)

### Fixed

- Fix unnecessary internal `py.allow_threads` GIL-switch when attempting to access contents of a `PyErr` which originated from Python (could lead to unintended deadlocks). [#4766](https://github.com/PyO3/pyo3/pull/4766)
- Fix thread-unsafe access of dict internals in `BoundDictIterator` on the free-threaded build. [#4788](https://github.com/PyO3/pyo3/pull/4788)
* Fix unnecessary critical sections in `BoundDictIterator` on the free-threaded build. [#4788](https://github.com/PyO3/pyo3/pull/4788)
- Fix time-of-check to time-of-use issues with list iteration on the free-threaded build. [#4789](https://github.com/PyO3/pyo3/pull/4789)
- Fix `chrono::DateTime<Tz>` to-Python conversion when `Tz` is `chrono_tz::Tz`. [#4790](https://github.com/PyO3/pyo3/pull/4790)
- Fix `#[pyclass]` not being able to be named `Probe`. [#4794](https://github.com/PyO3/pyo3/pull/4794)
- Fix not treating cross-compilation from x64 to aarch64 on Windows as a cross-compile. [#4800](https://github.com/PyO3/pyo3/pull/4800)
- Fix missing struct fields on GraalPy when subclassing builtin classes. [#4802](https://github.com/PyO3/pyo3/pull/4802)
- Fix generating import lib for PyPy when `abi3` feature is enabled. [#4806](https://github.com/PyO3/pyo3/pull/4806)
- Fix generating import lib for python3.13t when `abi3` feature is enabled. [#4808](https://github.com/PyO3/pyo3/pull/4808)
- Fix compile failure for raw identifiers like `r#box` in `derive(FromPyObject)`. [#4814](https://github.com/PyO3/pyo3/pull/4814)
- Fix compile failure for `#[pyclass]` enum variants with more than 12 fields. [#4832](https://github.com/PyO3/pyo3/pull/4832)


## [0.23.3] - 2024-12-03

### Packaging

- Bump optional `python3-dll-a` dependency to 0.2.11. [#4749](https://github.com/PyO3/pyo3/pull/4749)

### Fixed

- Fix unresolved symbol link failures on Windows when compiling for Python 3.13t with `abi3` features enabled. [#4733](https://github.com/PyO3/pyo3/pull/4733)
- Fix unresolved symbol link failures on Windows when compiling for Python 3.13t using the `generate-import-lib` feature. [#4749](https://github.com/PyO3/pyo3/pull/4749)
- Fix compile-time regression in PyO3 0.23.0 where changing `PYO3_CONFIG_FILE` would not reconfigure PyO3 for the new interpreter. [#4758](https://github.com/PyO3/pyo3/pull/4758)

## [0.23.2] - 2024-11-25

### Added

- Add `IntoPyObjectExt` trait. [#4708](https://github.com/PyO3/pyo3/pull/4708)

### Fixed

- Fix compile failures when building for free-threaded Python when the `abi3` or `abi3-pyxx` features are enabled. [#4719](https://github.com/PyO3/pyo3/pull/4719)
- Fix `ambiguous_associated_items` lint error in `#[pyclass]` and `#[derive(IntoPyObject)]` macros. [#4725](https://github.com/PyO3/pyo3/pull/4725)


## [0.23.1] - 2024-11-16

Re-release of 0.23.0 with fixes to docs.rs build.

## [0.23.0] - 2024-11-15

### Packaging

- Drop support for PyPy 3.7 and 3.8. [#4582](https://github.com/PyO3/pyo3/pull/4582)
- Extend range of supported versions of `hashbrown` optional dependency to include version 0.15. [#4604](https://github.com/PyO3/pyo3/pull/4604)
- Bump minimum version of `eyre` optional dependency to 0.6.8. [#4617](https://github.com/PyO3/pyo3/pull/4617)
- Bump minimum version of `hashbrown` optional dependency to 0.14.5. [#4617](https://github.com/PyO3/pyo3/pull/4617)
- Bump minimum version of `indexmap` optional dependency to 2.5.0. [#4617](https://github.com/PyO3/pyo3/pull/4617)
- Bump minimum version of `num-complex` optional dependency to 0.4.6. [#4617](https://github.com/PyO3/pyo3/pull/4617)
- Bump minimum version of `chrono-tz` optional dependency to 0.10. [#4617](https://github.com/PyO3/pyo3/pull/4617)
- Support free-threaded Python 3.13t. [#4588](https://github.com/PyO3/pyo3/pull/4588)

### Added

- Add `IntoPyObject` (fallible) conversion trait to convert from Rust to Python values. [#4060](https://github.com/PyO3/pyo3/pull/4060)
- Add `#[pyclass(str="<format string>")]` option to generate `__str__` based on a `Display` implementation or format string. [#4233](https://github.com/PyO3/pyo3/pull/4233)
- Implement `PartialEq` for `Bound<'py, PyInt>` with `u8`, `u16`, `u32`, `u64`, `u128`, `usize`, `i8`, `i16`, `i32`, `i64`, `i128` and `isize`. [#4317](https://github.com/PyO3/pyo3/pull/4317)
- Implement `PartialEq<f64>` and `PartialEq<f32>` for `Bound<'py, PyFloat>`. [#4348](https://github.com/PyO3/pyo3/pull/4348)
- Add `as_super` and `into_super` methods for `Bound<T: PyClass>`. [#4351](https://github.com/PyO3/pyo3/pull/4351)
- Add FFI definitions `PyCFunctionFast` and `PyCFunctionFastWithKeywords` [#4415](https://github.com/PyO3/pyo3/pull/4415)
- Add FFI definitions for `PyMutex` on Python 3.13 and newer. [#4421](https://github.com/PyO3/pyo3/pull/4421)
- Add `PyDict::locked_for_each` to iterate efficiently on freethreaded Python. [#4439](https://github.com/PyO3/pyo3/pull/4439)
- Add FFI definitions `PyObject_GetOptionalAttr`, `PyObject_GetOptionalAttrString`, `PyObject_HasAttrWithError`, `PyObject_HasAttrStringWithError`, `Py_CONSTANT_*` constants, `Py_GetConstant`, `Py_GetConstantBorrowed`, and `PyType_GetModuleByDef` on Python 3.13 and newer. [#4447](https://github.com/PyO3/pyo3/pull/4447)
- Add FFI definitions for the Python critical section API available on Python 3.13 and newer. [#4477](https://github.com/PyO3/pyo3/pull/4477)
- Add derive macro for `IntoPyObject`. [#4495](https://github.com/PyO3/pyo3/pull/4495)
- Add `Borrowed::as_ptr`. [#4520](https://github.com/PyO3/pyo3/pull/4520)
- Add FFI definition for `PyImport_AddModuleRef`. [#4529](https://github.com/PyO3/pyo3/pull/4529)
- Add `PyAnyMethods::try_iter`. [#4553](https://github.com/PyO3/pyo3/pull/4553)
- Add `pyo3::sync::with_critical_section`, a wrapper around the Python Critical Section API added in Python 3.13. [#4587](https://github.com/PyO3/pyo3/pull/4587)
- Add `#[pymodule(gil_used = false)]` option to declare that a module supports the free-threaded build. [#4588](https://github.com/PyO3/pyo3/pull/4588)
- Add `PyModule::gil_used` method to declare that a module supports the free-threaded build. [#4588](https://github.com/PyO3/pyo3/pull/4588)
- Add FFI definition `PyDateTime_CAPSULE_NAME`. [#4634](https://github.com/PyO3/pyo3/pull/4634)
- Add `PyMappingProxy` type to represent the `mappingproxy` Python class. [#4644](https://github.com/PyO3/pyo3/pull/4644)
- Add FFI definitions `PyList_Extend` and `PyList_Clear`. [#4667](https://github.com/PyO3/pyo3/pull/4667)
- Add derive macro for `IntoPyObjectRef`. [#4674](https://github.com/PyO3/pyo3/pull/4674)
- Add `pyo3::sync::OnceExt` and `pyo3::sync::OnceLockExt` traits. [#4676](https://github.com/PyO3/pyo3/pull/4676)

### Changed

- Prefer `IntoPyObject` over `IntoPy<Py<PyAny>>>` for `#[pyfunction]` and `#[pymethods]` return types. [#4060](https://github.com/PyO3/pyo3/pull/4060)
- Report multiple errors from `#[pyclass]` and `#[pyo3(..)]` attributes. [#4243](https://github.com/PyO3/pyo3/pull/4243)
- Nested declarative `#[pymodule]` are automatically treated as submodules (no `PyInit_` entrypoint is created). [#4308](https://github.com/PyO3/pyo3/pull/4308)
- Deprecate `PyAnyMethods::is_ellipsis` (`Py::is_ellipsis` was deprecated in PyO3 0.20). [#4322](https://github.com/PyO3/pyo3/pull/4322)
- Deprecate `PyLong` in favor of `PyInt`. [#4347](https://github.com/PyO3/pyo3/pull/4347)
- Rename `IntoPyDict::into_py_dict_bound` to `IntoPyDict::into_py_dict`. [#4388](https://github.com/PyO3/pyo3/pull/4388)
- `PyModule::from_code` now expects `&CStr` as arguments instead of `&str`. [#4404](https://github.com/PyO3/pyo3/pull/4404)
- Use "fastcall" Python calling convention for `#[pyfunction]`s when compiling on abi3 for Python 3.10 and up. [#4415](https://github.com/PyO3/pyo3/pull/4415)
- Remove `Copy` and `Clone` from `PyObject` struct FFI definition. [#4434](https://github.com/PyO3/pyo3/pull/4434)
- `Python::eval` and `Python::run` now take a `&CStr` instead of `&str`. [#4435](https://github.com/PyO3/pyo3/pull/4435)
- Deprecate `IPowModulo`, `PyClassAttributeDef`, `PyGetterDef`, `PyMethodDef`, `PyMethodDefType`, and `PySetterDef` from PyO3's public API. [#4441](https://github.com/PyO3/pyo3/pull/4441)
- `IntoPyObject` impls for `Vec<u8>`, `&[u8]`, `[u8; N]`, `Cow<[u8]>` and `SmallVec<[u8; N]>` now convert into Python `bytes` rather than a `list` of integers. [#4442](https://github.com/PyO3/pyo3/pull/4442)
- Emit a compile-time error when attempting to subclass a class that doesn't allow subclassing. [#4453](https://github.com/PyO3/pyo3/pull/4453)
- `IntoPyDict::into_py_dict` is now fallible due to `IntoPyObject` migration. [#4493](https://github.com/PyO3/pyo3/pull/4493)
- The `abi3` feature will now override config files provided via `PYO3_BUILD_CONFIG`. [#4497](https://github.com/PyO3/pyo3/pull/4497)
- Disable the `GILProtected` struct on free-threaded Python. [#4504](https://github.com/PyO3/pyo3/pull/4504)
- Updated FFI definitions for functions and struct fields that have been deprecated or removed from CPython. [#4534](https://github.com/PyO3/pyo3/pull/4534)
- Disable `PyListMethods::get_item_unchecked` on free-threaded Python. [#4539](https://github.com/PyO3/pyo3/pull/4539)
- Add `GILOnceCell::import`. [#4542](https://github.com/PyO3/pyo3/pull/4542)
- Deprecate `PyAnyMethods::iter` in favour of `PyAnyMethods::try_iter`. [#4553](https://github.com/PyO3/pyo3/pull/4553)
- The `#[pyclass]` macro now requires a types to be `Sync`. (Except for `#[pyclass(unsendable)]` types). [#4566](https://github.com/PyO3/pyo3/pull/4566)
- `PyList::new` and `PyTuple::new` are now fallible due to `IntoPyObject` migration. [#4580](https://github.com/PyO3/pyo3/pull/4580)
- `PyErr::matches` is now fallible due to `IntoPyObject` migration. [#4595](https://github.com/PyO3/pyo3/pull/4595)
- Deprecate `ToPyObject` in favour of `IntoPyObject` [#4595](https://github.com/PyO3/pyo3/pull/4595)
- Deprecate `PyWeakrefMethods::get_option`. [#4597](https://github.com/PyO3/pyo3/pull/4597)
- Seal `PyWeakrefMethods` trait. [#4598](https://github.com/PyO3/pyo3/pull/4598)
- Remove `PyNativeTypeInitializer` and `PyObjectInit` from the PyO3 public API. [#4611](https://github.com/PyO3/pyo3/pull/4611)
- Deprecate `IntoPy` in favor of `IntoPyObject` [#4618](https://github.com/PyO3/pyo3/pull/4618)
- Eagerly normalize exceptions in `PyErr::take()` and `PyErr::fetch()` on Python 3.11 and older. [#4655](https://github.com/PyO3/pyo3/pull/4655)
- Move `IntoPy::type_output` to `IntoPyObject::type_output`. [#4657](https://github.com/PyO3/pyo3/pull/4657)
- Change return type of `PyMapping::keys`, `PyMapping::values` and `PyMapping::items` to `Bound<'py, PyList>` instead of `Bound<'py, PySequence>`. [#4661](https://github.com/PyO3/pyo3/pull/4661)
- Complex enums now allow field types that either implement `IntoPyObject` by reference or by value together with `Clone`. This makes `Py<T>` available as field type. [#4694](https://github.com/PyO3/pyo3/pull/4694)


### Removed

- Remove all functionality deprecated in PyO3 0.20. [#4322](https://github.com/PyO3/pyo3/pull/4322)
- Remove all functionality deprecated in PyO3 0.21. [#4323](https://github.com/PyO3/pyo3/pull/4323)
- Deprecate `PyUnicode` in favour of `PyString`. [#4370](https://github.com/PyO3/pyo3/pull/4370)
- Remove deprecated `gil-refs` feature. [#4378](https://github.com/PyO3/pyo3/pull/4378)
- Remove private FFI definitions `_Py_IMMORTAL_REFCNT`, `_Py_IsImmortal`, `_Py_TPFLAGS_STATIC_BUILTIN`, `_Py_Dealloc`, `_Py_IncRef`, `_Py_DecRef`. [#4447](https://github.com/PyO3/pyo3/pull/4447)
- Remove private FFI definitions `_Py_c_sum`, `_Py_c_diff`, `_Py_c_neg`, `_Py_c_prod`, `_Py_c_quot`, `_Py_c_pow`, `_Py_c_abs`. [#4521](https://github.com/PyO3/pyo3/pull/4521)
- Remove `_borrowed` methods of `PyWeakRef` and `PyWeakRefProxy`. [#4528](https://github.com/PyO3/pyo3/pull/4528)
- Removed private FFI definition `_PyErr_ChainExceptions`. [#4534](https://github.com/PyO3/pyo3/pull/4534)

### Fixed

- Fix invalid library search path `lib_dir` when cross-compiling. [#4389](https://github.com/PyO3/pyo3/pull/4389)
- Fix FFI definition `Py_Is` for PyPy on 3.10 to call the function defined by PyPy. [#4447](https://github.com/PyO3/pyo3/pull/4447)
- Fix compile failure when using `#[cfg]` attributes for simple enum variants. [#4509](https://github.com/PyO3/pyo3/pull/4509)
- Fix compiler warning for `non_snake_case` method names inside `#[pymethods]` generated code. [#4567](https://github.com/PyO3/pyo3/pull/4567)
- Fix compile error with `#[derive(FromPyObject)]` generic struct with trait bounds. [#4645](https://github.com/PyO3/pyo3/pull/4645)
- Fix compile error for `#[classmethod]` and `#[staticmethod]` on magic methods. [#4654](https://github.com/PyO3/pyo3/pull/4654)
- Fix compile warning for `unsafe_op_in_unsafe_fn` in generated macro code. [#4674](https://github.com/PyO3/pyo3/pull/4674)
- Fix incorrect deprecation warning for `#[pyclass] enum`s with custom `__eq__` implementation. [#4692](https://github.com/PyO3/pyo3/pull/4692)
- Fix `non_upper_case_globals` lint firing for generated `__match_args__` on complex enums. [#4705](https://github.com/PyO3/pyo3/pull/4705)

## [0.22.5] - 2024-10-15

### Fixed

- Fix regression in 0.22.4 of naming collision in `__clear__` slot and `clear` method generated code. [#4619](https://github.com/PyO3/pyo3/pull/4619)


## [0.22.4] - 2024-10-12

### Added

- Add FFI definition `PyWeakref_GetRef` and `compat::PyWeakref_GetRef`. [#4528](https://github.com/PyO3/pyo3/pull/4528)

### Changed

- Deprecate `_borrowed` methods on `PyWeakRef` and `PyWeakrefProxy` (just use the owning forms). [#4590](https://github.com/PyO3/pyo3/pull/4590)

### Fixed

- Revert removal of private FFI function `_PyLong_NumBits` on Python 3.13 and later. [#4450](https://github.com/PyO3/pyo3/pull/4450)
- Fix `__traverse__` functions for base classes not being called by subclasses created with `#[pyclass(extends = ...)]`. [#4563](https://github.com/PyO3/pyo3/pull/4563)
- Fix regression in 0.22.3 failing compiles under `#![forbid(unsafe_code)]`. [#4574](https://github.com/PyO3/pyo3/pull/4574)
- Fix `create_exception` macro triggering lint and compile errors due to interaction with `gil-refs` feature. [#4589](https://github.com/PyO3/pyo3/pull/4589)
- Workaround possible use-after-free in `_borrowed` methods on `PyWeakRef` and `PyWeakrefProxy` by leaking their contents. [#4590](https://github.com/PyO3/pyo3/pull/4590)
- Fix crash calling `PyType_GetSlot` on static types before Python 3.10. [#4599](https://github.com/PyO3/pyo3/pull/4599)


## [0.22.3] - 2024-09-15

### Added

- Add `pyo3::ffi::compat` namespace with compatibility shims for C API functions added in recent versions of Python.
- Add FFI definition `PyDict_GetItemRef` on Python 3.13 and newer, and `compat::PyDict_GetItemRef` for all versions. [#4355](https://github.com/PyO3/pyo3/pull/4355)
- Add FFI definition `PyList_GetItemRef` on Python 3.13 and newer, and `pyo3_ffi::compat::PyList_GetItemRef` for all versions. [#4410](https://github.com/PyO3/pyo3/pull/4410)
- Add FFI definitions `compat::Py_NewRef` and `compat::Py_XNewRef`. [#4445](https://github.com/PyO3/pyo3/pull/4445)
- Add FFI definitions `compat::PyObject_CallNoArgs` and `compat::PyObject_CallMethodNoArgs`. [#4461](https://github.com/PyO3/pyo3/pull/4461)
- Add `GilOnceCell<Py<T>>::clone_ref`. [#4511](https://github.com/PyO3/pyo3/pull/4511)

### Changed

- Improve error messages for `#[pyfunction]` defined inside `#[pymethods]`. [#4349](https://github.com/PyO3/pyo3/pull/4349)
- Improve performance of calls to Python by using the vectorcall calling convention where possible. [#4456](https://github.com/PyO3/pyo3/pull/4456)
- Mention the type name in the exception message when trying to instantiate a class with no constructor defined. [#4481](https://github.com/PyO3/pyo3/pull/4481)

### Removed

- Remove private FFI definition `_Py_PackageContext`. [#4420](https://github.com/PyO3/pyo3/pull/4420)

### Fixed

- Fix compile failure in declarative `#[pymodule]` under presence of `#![no_implicit_prelude]`. [#4328](https://github.com/PyO3/pyo3/pull/4328)
- Fix use of borrowed reference in `PyDict::get_item` (unsafe in free-threaded Python). [#4355](https://github.com/PyO3/pyo3/pull/4355)
- Fix `#[pyclass(eq)]` macro hygiene issues for structs and enums. [#4359](https://github.com/PyO3/pyo3/pull/4359)
- Fix hygiene/span issues of `#[pyfunction]` and `#[pymethods]` generated code which affected expansion in `macro_rules` context. [#4382](https://github.com/PyO3/pyo3/pull/4382)
- Fix `unsafe_code` lint error in `#[pyclass]` generated code. [#4396](https://github.com/PyO3/pyo3/pull/4396)
- Fix async functions returning a tuple only returning the first element to Python. [#4407](https://github.com/PyO3/pyo3/pull/4407)
- Fix use of borrowed reference in `PyList::get_item` (unsafe in free-threaded Python). [#4410](https://github.com/PyO3/pyo3/pull/4410)
- Correct FFI definition `PyArg_ParseTupleAndKeywords` to take `*const *const c_char` instead of `*mut *mut c_char` on Python 3.13 and up. [#4420](https://github.com/PyO3/pyo3/pull/4420)
- Fix a soundness bug with `PyClassInitializer`: panic if adding subclass to existing instance via `PyClassInitializer::from(Py<BaseClass>).add_subclass(SubClass)`. [#4454](https://github.com/PyO3/pyo3/pull/4454)
- Fix illegal reference counting op inside implementation of `__traverse__` handlers. [#4479](https://github.com/PyO3/pyo3/pull/4479)

## [0.22.2] - 2024-07-17

### Packaging

- Require opt-in to freethreaded Python using the `UNSAFE_PYO3_BUILD_FREE_THREADED=1` environment variable (it is not yet supported by PyO3). [#4327](https://github.com/PyO3/pyo3/pull/4327)

### Changed

- Use FFI function calls for reference counting on all abi3 versions. [#4324](https://github.com/PyO3/pyo3/pull/4324)
- `#[pymodule(...)]` now directly accepts all relevant `#[pyo3(...)]` options. [#4330](https://github.com/PyO3/pyo3/pull/4330)

### Fixed

- Fix compile failure in declarative `#[pymodule]` under presence of `#![no_implicit_prelude]`. [#4328](https://github.com/PyO3/pyo3/pull/4328)
- Fix compile failure due to c-string literals on Rust < 1.79. [#4353](https://github.com/PyO3/pyo3/pull/4353)

## [0.22.1] - 2024-07-06

### Added

- Add `#[pyo3(submodule)]` option for declarative `#[pymodule]`s. [#4301](https://github.com/PyO3/pyo3/pull/4301)
- Implement `PartialEq<bool>` for `Bound<'py, PyBool>`. [#4305](https://github.com/PyO3/pyo3/pull/4305)

### Fixed

- Return `NotImplemented` instead of raising `TypeError` from generated equality method when comparing different types. [#4287](https://github.com/PyO3/pyo3/pull/4287)
- Handle full-path `#[pyo3::prelude::pymodule]` and similar for `#[pyclass]` and `#[pyfunction]` in declarative modules. [#4288](https://github.com/PyO3/pyo3/pull/4288)
- Fix 128-bit int regression on big-endian platforms with Python <3.13. [#4291](https://github.com/PyO3/pyo3/pull/4291)
- Stop generating code that will never be covered with declarative modules. [#4297](https://github.com/PyO3/pyo3/pull/4297)
- Fix invalid deprecation warning for trailing optional on `#[setter]` function. [#4304](https://github.com/PyO3/pyo3/pull/4304)

## [0.22.0] - 2024-06-24

### Packaging

- Update `heck` dependency to 0.5. [#3966](https://github.com/PyO3/pyo3/pull/3966)
- Extend range of supported versions of `chrono-tz` optional dependency to include version 0.10. [#4061](https://github.com/PyO3/pyo3/pull/4061)
- Update MSRV to 1.63. [#4129](https://github.com/PyO3/pyo3/pull/4129)
- Add optional `num-rational` feature to add conversions with Python's `fractions.Fraction`. [#4148](https://github.com/PyO3/pyo3/pull/4148)
- Support Python 3.13. [#4184](https://github.com/PyO3/pyo3/pull/4184)

### Added

- Add `PyWeakref`, `PyWeakrefReference` and `PyWeakrefProxy`. [#3835](https://github.com/PyO3/pyo3/pull/3835)
- Support `#[pyclass]` on enums that have tuple variants. [#4072](https://github.com/PyO3/pyo3/pull/4072)
- Add support for scientific notation in `Decimal` conversion. [#4079](https://github.com/PyO3/pyo3/pull/4079)
- Add `pyo3_disable_reference_pool` conditional compilation flag to avoid the overhead of the global reference pool at the cost of known limitations as explained in the performance section of the guide. [#4095](https://github.com/PyO3/pyo3/pull/4095)
- Add `#[pyo3(constructor = (...))]` to customize the generated constructors for complex enum variants. [#4158](https://github.com/PyO3/pyo3/pull/4158)
- Add `PyType::module`, which always matches Python `__module__`. [#4196](https://github.com/PyO3/pyo3/pull/4196)
- Add `PyType::fully_qualified_name` which matches the "fully qualified name" defined in [PEP 737](https://peps.python.org/pep-0737). [#4196](https://github.com/PyO3/pyo3/pull/4196)
- Add `PyTypeMethods::mro` and `PyTypeMethods::bases`. [#4197](https://github.com/PyO3/pyo3/pull/4197)
- Add `#[pyclass(ord)]` to implement ordering based on `PartialOrd`. [#4202](https://github.com/PyO3/pyo3/pull/4202)
- Implement `ToPyObject` and `IntoPy<PyObject>` for `PyBackedStr` and `PyBackedBytes`. [#4205](https://github.com/PyO3/pyo3/pull/4205)
- Add `#[pyclass(hash)]` option to implement `__hash__` in terms of the `Hash` implementation [#4206](https://github.com/PyO3/pyo3/pull/4206)
- Add `#[pyclass(eq)]` option to generate `__eq__` based on `PartialEq`, and `#[pyclass(eq_int)]` for simple enums to implement equality based on their discriminants. [#4210](https://github.com/PyO3/pyo3/pull/4210)
- Implement `From<Bound<'py, T>>` for `PyClassInitializer<T>`. [#4214](https://github.com/PyO3/pyo3/pull/4214)
- Add `as_super` methods to `PyRef` and `PyRefMut` for accesing the base class by reference. [#4219](https://github.com/PyO3/pyo3/pull/4219)
- Implement `PartialEq<str>` for `Bound<'py, PyString>`. [#4245](https://github.com/PyO3/pyo3/pull/4245)
- Implement `PyModuleMethods::filename` on PyPy. [#4249](https://github.com/PyO3/pyo3/pull/4249)
- Implement `PartialEq<[u8]>` for `Bound<'py, PyBytes>`. [#4250](https://github.com/PyO3/pyo3/pull/4250)
- Add `pyo3_ffi::c_str` macro to create `&'static CStr` on Rust versions which don't have 1.77's `c""` literals. [#4255](https://github.com/PyO3/pyo3/pull/4255)
- Support `bool` conversion with `numpy` 2.0's `numpy.bool` type [#4258](https://github.com/PyO3/pyo3/pull/4258)
- Add `PyAnyMethods::{bitnot, matmul, floor_div, rem, divmod}`. [#4264](https://github.com/PyO3/pyo3/pull/4264)

### Changed

- Change the type of `PySliceIndices::slicelength` and the `length` parameter of `PySlice::indices()`. [#3761](https://github.com/PyO3/pyo3/pull/3761)
- Deprecate implicit default for trailing optional arguments [#4078](https://github.com/PyO3/pyo3/pull/4078)
- `Clone`ing pointers into the Python heap has been moved behind the `py-clone` feature, as it must panic without the GIL being held as a soundness fix. [#4095](https://github.com/PyO3/pyo3/pull/4095)
- Add `#[track_caller]` to all `Py<T>`, `Bound<'py, T>` and `Borrowed<'a, 'py, T>` methods which can panic. [#4098](https://github.com/PyO3/pyo3/pull/4098)
- Change `PyAnyMethods::dir` to be fallible and return `PyResult<Bound<'py, PyList>>` (and similar for `PyAny::dir`). [#4100](https://github.com/PyO3/pyo3/pull/4100)
- The global reference pool (to track pending reference count decrements) is now initialized lazily to avoid the overhead of taking a mutex upon function entry when the functionality is not actually used. [#4178](https://github.com/PyO3/pyo3/pull/4178)
- Emit error messages when using `weakref` or `dict` when compiling for `abi3` for Python older than 3.9. [#4194](https://github.com/PyO3/pyo3/pull/4194)
- Change `PyType::name` to always match Python `__name__`. [#4196](https://github.com/PyO3/pyo3/pull/4196)
- Remove CPython internal ffi call for complex number including: add, sub, mul, div, neg, abs, pow. Added PyAnyMethods::{abs, pos, neg} [#4201](https://github.com/PyO3/pyo3/pull/4201)
- Deprecate implicit integer comparision for simple enums in favor of `#[pyclass(eq_int)]`. [#4210](https://github.com/PyO3/pyo3/pull/4210)
- Set the `module=` attribute of declarative modules' child `#[pymodule]`s and `#[pyclass]`es. [#4213](https://github.com/PyO3/pyo3/pull/4213)
- Set the `module` option for complex enum variants from the value set on the complex enum `module`. [#4228](https://github.com/PyO3/pyo3/pull/4228)
- Respect the Python "limited API" when building for the `abi3` feature on PyPy or GraalPy. [#4237](https://github.com/PyO3/pyo3/pull/4237)
- Optimize code generated by `#[pyo3(get)]` on `#[pyclass]` fields. [#4254](https://github.com/PyO3/pyo3/pull/4254)
- `PyCFunction::new`, `PyCFunction::new_with_keywords` and `PyCFunction::new_closure` now take `&'static CStr` name and doc arguments (previously was `&'static str`). [#4255](https://github.com/PyO3/pyo3/pull/4255)
- The `experimental-declarative-modules` feature is now stabilized and available by default. [#4257](https://github.com/PyO3/pyo3/pull/4257)

### Fixed

- Fix panic when `PYO3_CROSS_LIB_DIR` is set to a missing path. [#4043](https://github.com/PyO3/pyo3/pull/4043)
- Fix a compile error when exporting an exception created with `create_exception!` living in a different Rust module using the `declarative-module` feature. [#4086](https://github.com/PyO3/pyo3/pull/4086)
- Fix FFI definitions of `PY_VECTORCALL_ARGUMENTS_OFFSET` and `PyVectorcall_NARGS` to fix a false-positive assertion. [#4104](https://github.com/PyO3/pyo3/pull/4104)
- Disable `PyUnicode_DATA` on PyPy: not exposed by PyPy. [#4116](https://github.com/PyO3/pyo3/pull/4116)
- Correctly handle `#[pyo3(from_py_with = ...)]` attribute on dunder (`__magic__`) method arguments instead of silently ignoring it. [#4117](https://github.com/PyO3/pyo3/pull/4117)
- Fix a compile error when declaring a standalone function or class method with a Python name that is a Rust keyword. [#4226](https://github.com/PyO3/pyo3/pull/4226)
- Fix declarative modules discarding doc comments on the `mod` node. [#4236](https://github.com/PyO3/pyo3/pull/4236)
- Fix `__dict__` attribute missing for `#[pyclass(dict)]` instances when building for `abi3` on Python 3.9. [#4251](https://github.com/PyO3/pyo3/pull/4251)

## [0.21.2] - 2024-04-16

### Changed

- Deprecate the `PySet::empty()` gil-ref constructor. [#4082](https://github.com/PyO3/pyo3/pull/4082)

### Fixed

- Fix compile error for `async fn` in `#[pymethods]` with a `&self` receiver and more than one additional argument. [#4035](https://github.com/PyO3/pyo3/pull/4035)
- Improve error message for wrong receiver type in `__traverse__`. [#4045](https://github.com/PyO3/pyo3/pull/4045)
- Fix compile error when exporting a `#[pyclass]` living in a different Rust module using the `experimental-declarative-modules` feature. [#4054](https://github.com/PyO3/pyo3/pull/4054)
- Fix `missing_docs` lint triggering on documented `#[pymodule]` functions. [#4067](https://github.com/PyO3/pyo3/pull/4067)
- Fix undefined symbol errors for extension modules on AIX (by linking `libpython`). [#4073](https://github.com/PyO3/pyo3/pull/4073)

## [0.21.1] - 2024-04-01

### Added

- Implement `Send` and `Sync` for `PyBackedStr` and `PyBackedBytes`. [#4007](https://github.com/PyO3/pyo3/pull/4007)
- Implement `Clone`, `Debug`, `PartialEq`, `Eq`, `PartialOrd`, `Ord` and `Hash` implementation for `PyBackedBytes` and `PyBackedStr`, and `Display` for `PyBackedStr`. [#4020](https://github.com/PyO3/pyo3/pull/4020)
- Add `import_exception_bound!` macro to import exception types without generating GIL Ref functionality for them. [#4027](https://github.com/PyO3/pyo3/pull/4027)

### Changed

- Emit deprecation warning for uses of GIL Refs as `#[setter]` function arguments. [#3998](https://github.com/PyO3/pyo3/pull/3998)
- Add `#[inline]` hints on many `Bound` and `Borrowed` methods. [#4024](https://github.com/PyO3/pyo3/pull/4024)

### Fixed

- Handle `#[pyo3(from_py_with = "")]` in `#[setter]` methods [#3995](https://github.com/PyO3/pyo3/pull/3995)
- Allow extraction of `&Bound` in `#[setter]` methods. [#3998](https://github.com/PyO3/pyo3/pull/3998)
- Fix some uncovered code blocks emitted by `#[pymodule]`, `#[pyfunction]` and `#[pyclass]` macros. [#4009](https://github.com/PyO3/pyo3/pull/4009)
- Fix typo in the panic message when a class referenced in `pyo3::import_exception!` does not exist. [#4012](https://github.com/PyO3/pyo3/pull/4012)
- Fix compile error when using an async `#[pymethod]` with a receiver and additional arguments. [#4015](https://github.com/PyO3/pyo3/pull/4015)


## [0.21.0] - 2024-03-25

### Added

- Add support for GraalPy (24.0 and up). [#3247](https://github.com/PyO3/pyo3/pull/3247)
- Add `PyMemoryView` type. [#3514](https://github.com/PyO3/pyo3/pull/3514)
- Allow `async fn` in for `#[pyfunction]` and `#[pymethods]`, with the `experimental-async` feature. [#3540](https://github.com/PyO3/pyo3/pull/3540) [#3588](https://github.com/PyO3/pyo3/pull/3588) [#3599](https://github.com/PyO3/pyo3/pull/3599) [#3931](https://github.com/PyO3/pyo3/pull/3931)
- Implement `PyTypeInfo` for `PyEllipsis`, `PyNone` and `PyNotImplemented`. [#3577](https://github.com/PyO3/pyo3/pull/3577)
- Support `#[pyclass]` on enums that have non-unit variants. [#3582](https://github.com/PyO3/pyo3/pull/3582)
- Support `chrono` feature with `abi3` feature. [#3664](https://github.com/PyO3/pyo3/pull/3664)
- `FromPyObject`, `IntoPy<PyObject>` and `ToPyObject` are implemented on `std::duration::Duration` [#3670](https://github.com/PyO3/pyo3/pull/3670)
- Add `PyString::to_cow`. Add `Py<PyString>::to_str`, `Py<PyString>::to_cow`, and `Py<PyString>::to_string_lossy`, as ways to access Python string data safely beyond the GIL lifetime. [#3677](https://github.com/PyO3/pyo3/pull/3677)
- Add `Bound<T>` and `Borrowed<T>` smart pointers as a new API for accessing Python objects. [#3686](https://github.com/PyO3/pyo3/pull/3686)
- Add `PyNativeType::as_borrowed` to convert "GIL refs" to the new `Bound` smart pointer. [#3692](https://github.com/PyO3/pyo3/pull/3692)
- Add `FromPyObject::extract_bound` method, to migrate `FromPyObject` implementations to the Bound API. [#3706](https://github.com/PyO3/pyo3/pull/3706)
- Add `gil-refs` feature to allow continued use of the deprecated GIL Refs APIs. [#3707](https://github.com/PyO3/pyo3/pull/3707)
- Add methods to `PyAnyMethods` for binary operators (`add`, `sub`, etc.) [#3712](https://github.com/PyO3/pyo3/pull/3712)
- Add `chrono-tz` feature allowing conversion between `chrono_tz::Tz` and `zoneinfo.ZoneInfo` [#3730](https://github.com/PyO3/pyo3/pull/3730)
- Add FFI definition `PyType_GetModuleByDef`. [#3734](https://github.com/PyO3/pyo3/pull/3734)
- Conversion between `std::time::SystemTime` and `datetime.datetime` [#3736](https://github.com/PyO3/pyo3/pull/3736)
- Add `Py::as_any` and `Py::into_any`. [#3785](https://github.com/PyO3/pyo3/pull/3785)
- Add `PyStringMethods::encode_utf8`. [#3801](https://github.com/PyO3/pyo3/pull/3801)
- Add `PyBackedStr` and `PyBackedBytes`, as alternatives to `&str` and `&bytes` where a Python object owns the data. [#3802](https://github.com/PyO3/pyo3/pull/3802) [#3991](https://github.com/PyO3/pyo3/pull/3991)
- Allow `#[pymodule]` macro on Rust `mod` blocks, with the `experimental-declarative-modules` feature. [#3815](https://github.com/PyO3/pyo3/pull/3815)
- Implement `ExactSizeIterator` for `set` and `frozenset` iterators on `abi3` feature. [#3849](https://github.com/PyO3/pyo3/pull/3849)
- Add `Py::drop_ref` to explicitly drop a `Py`` and immediately decrease the Python reference count if the GIL is already held. [#3871](https://github.com/PyO3/pyo3/pull/3871)
- Allow `#[pymodule]` macro on single argument functions that take `&Bound<'_, PyModule>`. [#3905](https://github.com/PyO3/pyo3/pull/3905)
- Implement `FromPyObject` for `Cow<str>`. [#3928](https://github.com/PyO3/pyo3/pull/3928)
- Implement `Default` for `GILOnceCell`. [#3971](https://github.com/PyO3/pyo3/pull/3971)
- Add `PyDictMethods::into_mapping`, `PyListMethods::into_sequence` and `PyTupleMethods::into_sequence`. [#3982](https://github.com/PyO3/pyo3/pull/3982)

### Changed

- `PyDict::from_sequence` now takes a single argument of type `&PyAny` (previously took two arguments `Python` and `PyObject`). [#3532](https://github.com/PyO3/pyo3/pull/3532)
- Deprecate `Py::is_ellipsis` and `PyAny::is_ellipsis` in favour of `any.is(py.Ellipsis())`. [#3577](https://github.com/PyO3/pyo3/pull/3577)
- Split some `PyTypeInfo` functionality into new traits `HasPyGilRef` and `PyTypeCheck`. [#3600](https://github.com/PyO3/pyo3/pull/3600)
- Deprecate `PyTryFrom` and `PyTryInto` traits in favor of `any.downcast()` via the `PyTypeCheck` and `PyTypeInfo` traits. [#3601](https://github.com/PyO3/pyo3/pull/3601)
- Allow async methods to accept `&self`/`&mut self` [#3609](https://github.com/PyO3/pyo3/pull/3609)
- `FromPyObject` for set types now also accept `frozenset` objects as input. [#3632](https://github.com/PyO3/pyo3/pull/3632)
- `FromPyObject` for `bool` now also accepts NumPy's `bool_` as input. [#3638](https://github.com/PyO3/pyo3/pull/3638)
- Add `AsRefSource` associated type to `PyNativeType`. [#3653](https://github.com/PyO3/pyo3/pull/3653)
- Rename `.is_true` to `.is_truthy` on `PyAny` and `Py<PyAny>` to clarify that the test is not based on identity with or equality to the True singleton. [#3657](https://github.com/PyO3/pyo3/pull/3657)
- `PyType::name` is now `PyType::qualname` whereas `PyType::name` efficiently accesses the full name which includes the module name. [#3660](https://github.com/PyO3/pyo3/pull/3660)
- The `Iter(A)NextOutput` types are now deprecated and `__(a)next__` can directly return anything which can be converted into Python objects, i.e. awaitables do not need to be wrapped into `IterANextOutput` or `Option` any more. `Option` can still be used as well and returning `None` will trigger the fast path for `__next__`, stopping iteration without having to raise a `StopIteration` exception. [#3661](https://github.com/PyO3/pyo3/pull/3661)
- Implement `FromPyObject` on `chrono::DateTime<Tz>` for all `Tz`, not just `FixedOffset` and `Utc`. [#3663](https://github.com/PyO3/pyo3/pull/3663)
- Add lifetime parameter to `PyTzInfoAccess` trait. For the deprecated gil-ref API, the trait is now implemented for `&'py PyTime` and `&'py PyDateTime` instead of `PyTime` and `PyDate`. [#3679](https://github.com/PyO3/pyo3/pull/3679)
- Calls to `__traverse__` become no-ops for unsendable pyclasses if on the wrong thread, thereby avoiding hard aborts at the cost of potential leakage. [#3689](https://github.com/PyO3/pyo3/pull/3689)
- Include `PyNativeType` in `pyo3::prelude`. [#3692](https://github.com/PyO3/pyo3/pull/3692)
- Improve performance of `extract::<i64>` (and other integer types) by avoiding call to `__index__()` converting the value to an integer for 3.10+. Gives performance improvement of around 30% for successful extraction. [#3742](https://github.com/PyO3/pyo3/pull/3742)
- Relax bound of `FromPyObject` for `Py<T>` to just `T: PyTypeCheck`. [#3776](https://github.com/PyO3/pyo3/pull/3776)
- `PySet` and `PyFrozenSet` iterators now always iterate the equivalent of `iter(set)`. (A "fast path" with no noticeable performance benefit was removed.) [#3849](https://github.com/PyO3/pyo3/pull/3849)
- Move implementations of `FromPyObject` for `&str`, `Cow<str>`, `&[u8]` and `Cow<[u8]>` onto a temporary trait `FromPyObjectBound` when `gil-refs` feature is deactivated. [#3928](https://github.com/PyO3/pyo3/pull/3928)
- Deprecate `GILPool`, `Python::with_pool`, and `Python::new_pool`. [#3947](https://github.com/PyO3/pyo3/pull/3947)

### Removed

- Remove all functionality deprecated in PyO3 0.19. [#3603](https://github.com/PyO3/pyo3/pull/3603)

### Fixed

- Match PyPy 7.3.14 in removing PyPy-only symbol `Py_MAX_NDIMS` in favour of `PyBUF_MAX_NDIM`. [#3757](https://github.com/PyO3/pyo3/pull/3757)
- Fix segmentation fault using `datetime` types when an invalid `datetime` module is on sys.path. [#3818](https://github.com/PyO3/pyo3/pull/3818)
- Fix `non_local_definitions` lint warning triggered by many PyO3 macros. [#3901](https://github.com/PyO3/pyo3/pull/3901)
- Disable `PyCode` and `PyCode_Type` on PyPy: `PyCode_Type` is not exposed by PyPy. [#3934](https://github.com/PyO3/pyo3/pull/3934)

## [0.21.0-beta.0] - 2024-03-10

Prerelease of PyO3 0.21. See [the GitHub diff](https://github.com/pyo3/pyo3/compare/v0.21.0-beta.0...v0.21.0) for what changed between 0.21.0-beta.0 and the final release.

## [0.20.3] - 2024-02-23

### Packaging

- Add `portable-atomic` dependency. [#3619](https://github.com/PyO3/pyo3/pull/3619)
- Check maximum version of Python at build time and for versions not yet supported require opt-in to the `abi3` stable ABI by the environment variable `PYO3_USE_ABI3_FORWARD_COMPATIBILITY=1`. [#3821](https://github.com/PyO3/pyo3/pull/3821)

### Fixed

- Use `portable-atomic` to support platforms without 64-bit atomics. [#3619](https://github.com/PyO3/pyo3/pull/3619)
- Fix compilation failure with `either` feature enabled without `experimental-inspect` enabled. [#3834](https://github.com/PyO3/pyo3/pull/3834)

## [0.20.2] - 2024-01-04

### Packaging

- Pin `pyo3` and `pyo3-ffi` dependencies on `pyo3-build-config` to require the same patch version, i.e. `pyo3` 0.20.2 requires _exactly_ `pyo3-build-config` 0.20.2. [#3721](https://github.com/PyO3/pyo3/pull/3721)

### Fixed

- Fix compile failure when building `pyo3` 0.20.0 with latest `pyo3-build-config` 0.20.X. [#3724](https://github.com/PyO3/pyo3/pull/3724)
- Fix docs.rs build. [#3722](https://github.com/PyO3/pyo3/pull/3722)

## [0.20.1] - 2023-12-30

### Added

- Add optional `either` feature to add conversions for `either::Either<L, R>` sum type. [#3456](https://github.com/PyO3/pyo3/pull/3456)
- Add optional `smallvec` feature to add conversions for `smallvec::SmallVec`. [#3507](https://github.com/PyO3/pyo3/pull/3507)
- Add `take` and `into_inner` methods to `GILOnceCell` [#3556](https://github.com/PyO3/pyo3/pull/3556)
- `#[classmethod]` methods can now also receive `Py<PyType>` as their first argument. [#3587](https://github.com/PyO3/pyo3/pull/3587)
- `#[pyfunction(pass_module)]` can now also receive `Py<PyModule>` as their first argument. [#3587](https://github.com/PyO3/pyo3/pull/3587)
- Add `traverse` method to `GILProtected`. [#3616](https://github.com/PyO3/pyo3/pull/3616)
- Added `abi3-py312` feature [#3687](https://github.com/PyO3/pyo3/pull/3687)

### Fixed

- Fix minimum version specification for optional `chrono` dependency. [#3512](https://github.com/PyO3/pyo3/pull/3512)
- Silenced new `clippy::unnecessary_fallible_conversions` warning when using a `Py<Self>` `self` receiver. [#3564](https://github.com/PyO3/pyo3/pull/3564)


## [0.20.0] - 2023-10-11

### Packaging

- Dual-license PyO3 under either the Apache 2.0 OR the MIT license. This makes the project GPLv2 compatible. [#3108](https://github.com/PyO3/pyo3/pull/3108)
- Update MSRV to Rust 1.56. [#3208](https://github.com/PyO3/pyo3/pull/3208)
- Bump `indoc` dependency to 2.0 and `unindent` dependency to 0.2. [#3237](https://github.com/PyO3/pyo3/pull/3237)
- Bump `syn` dependency to 2.0. [#3239](https://github.com/PyO3/pyo3/pull/3239)
- Drop support for debug builds of Python 3.7. [#3387](https://github.com/PyO3/pyo3/pull/3387)
- Bump `chrono` optional dependency to require 0.4.25 or newer. [#3427](https://github.com/PyO3/pyo3/pull/3427)
- Support Python 3.12. [#3488](https://github.com/PyO3/pyo3/pull/3488)

### Added

- Support `__lt__`, `__le__`, `__eq__`, `__ne__`, `__gt__` and `__ge__` in `#[pymethods]`. [#3203](https://github.com/PyO3/pyo3/pull/3203)
- Add FFI definition `Py_GETENV`. [#3336](https://github.com/PyO3/pyo3/pull/3336)
- Add `as_ptr` and `into_ptr` inherent methods for `Py`, `PyAny`, `PyRef`, and `PyRefMut`. [#3359](https://github.com/PyO3/pyo3/pull/3359)
- Implement `DoubleEndedIterator` for `PyTupleIterator` and `PyListIterator`. [#3366](https://github.com/PyO3/pyo3/pull/3366)
- Add `#[pyclass(rename_all = "...")]` option: this allows renaming all getters and setters of a struct, or all variants of an enum. Available renaming rules are: `"camelCase"`, `"kebab-case"`, `"lowercase"`, `"PascalCase"`, `"SCREAMING-KEBAB-CASE"`, `"SCREAMING_SNAKE_CASE"`, `"snake_case"`, `"UPPERCASE"`. [#3384](https://github.com/PyO3/pyo3/pull/3384)
- Add FFI definitions `PyObject_GC_IsTracked` and `PyObject_GC_IsFinalized` on Python 3.9 and up (PyPy 3.10 and up). [#3403](https://github.com/PyO3/pyo3/pull/3403)
- Add types for `None`, `Ellipsis`, and `NotImplemented`. [#3408](https://github.com/PyO3/pyo3/pull/3408)
- Add FFI definitions for the `Py_mod_multiple_interpreters` constant and its possible values. [#3494](https://github.com/PyO3/pyo3/pull/3494)
- Add FFI definitions for `PyInterpreterConfig` struct, its constants and `Py_NewInterpreterFromConfig`. [#3502](https://github.com/PyO3/pyo3/pull/3502)

### Changed

- Change `PySet::discard` to return `PyResult<bool>` (previously returned nothing). [#3281](https://github.com/PyO3/pyo3/pull/3281)
- Optimize implmentation of `IntoPy` for Rust tuples to Python tuples. [#3321](https://github.com/PyO3/pyo3/pull/3321)
- Change `PyDict::get_item` to no longer suppress arbitrary exceptions (the return type is now `PyResult<Option<&PyAny>>` instead of `Option<&PyAny>`), and deprecate `PyDict::get_item_with_error`. [#3330](https://github.com/PyO3/pyo3/pull/3330)
- Deprecate FFI definitions which are deprecated in Python 3.12. [#3336](https://github.com/PyO3/pyo3/pull/3336)
- `AsPyPointer` is now an `unsafe trait`. [#3358](https://github.com/PyO3/pyo3/pull/3358)
- Accept all `os.PathLike` values in implementation of `FromPyObject` for `PathBuf`. [#3374](https://github.com/PyO3/pyo3/pull/3374)
- Add `__builtins__` to globals in `py.run()` and `py.eval()` if they're missing. [#3378](https://github.com/PyO3/pyo3/pull/3378)
- Optimize implementation of `FromPyObject` for `BigInt` and `BigUint`. [#3379](https://github.com/PyO3/pyo3/pull/3379)
- `PyIterator::from_object` and `PyByteArray::from` now take a single argument of type `&PyAny` (previously took two arguments `Python` and `AsPyPointer`). [#3389](https://github.com/PyO3/pyo3/pull/3389)
- Replace `AsPyPointer` with `AsRef<PyAny>` as a bound in the blanket implementation of `From<&T> for PyObject`. [#3391](https://github.com/PyO3/pyo3/pull/3391)
- Replace blanket `impl IntoPy<PyObject> for &T where T: AsPyPointer` with implementations of `impl IntoPy<PyObject>` for `&PyAny`, `&T where T: AsRef<PyAny>`, and `&Py<T>`. [#3393](https://github.com/PyO3/pyo3/pull/3393)
- Preserve `std::io::Error` kind in implementation of `From<std::io::IntoInnerError>` for `PyErr` [#3396](https://github.com/PyO3/pyo3/pull/3396)
- Try to select a relevant `ErrorKind` in implementation of `From<PyErr>` for `OSError` subclass. [#3397](https://github.com/PyO3/pyo3/pull/3397)
- Retrieve the original `PyErr` in implementation of `From<std::io::Error>` for `PyErr` if the `std::io::Error` has been built using a Python exception (previously would create a new exception wrapping the `std::io::Error`). [#3402](https://github.com/PyO3/pyo3/pull/3402)
- `#[pymodule]` will now return the same module object on repeated import by the same Python interpreter, on Python 3.9 and up. [#3446](https://github.com/PyO3/pyo3/pull/3446)
- Truncate leap-seconds and warn when converting `chrono` types to Python `datetime` types (`datetime` cannot represent leap-seconds). [#3458](https://github.com/PyO3/pyo3/pull/3458)
- `Err` returned from `#[pyfunction]` will now have a non-None `__context__` if called from inside a `catch` block. [#3455](https://github.com/PyO3/pyo3/pull/3455)
- Deprecate undocumented `#[__new__]` form of `#[new]` attribute. [#3505](https://github.com/PyO3/pyo3/pull/3505)

### Removed

- Remove all functionality deprecated in PyO3 0.18, including `#[args]` attribute for `#[pymethods]`. [#3232](https://github.com/PyO3/pyo3/pull/3232)
- Remove `IntoPyPointer` trait in favour of `into_ptr` inherent methods. [#3385](https://github.com/PyO3/pyo3/pull/3385)

### Fixed

- Handle exceptions properly in `PySet::discard`. [#3281](https://github.com/PyO3/pyo3/pull/3281)
- The `PyTupleIterator` type returned by `PyTuple::iter` is now public and hence can be named by downstream crates. [#3366](https://github.com/PyO3/pyo3/pull/3366)
- Linking of `PyOS_FSPath` on PyPy. [#3374](https://github.com/PyO3/pyo3/pull/3374)
- Fix memory leak in `PyTypeBuilder::build`. [#3401](https://github.com/PyO3/pyo3/pull/3401)
- Disable removed FFI definitions `_Py_GetAllocatedBlocks`, `_PyObject_GC_Malloc`, and `_PyObject_GC_Calloc` on Python 3.11 and up. [#3403](https://github.com/PyO3/pyo3/pull/3403)
- Fix `ResourceWarning` and crashes related to GC when running with debug builds of CPython. [#3404](https://github.com/PyO3/pyo3/pull/3404)
- Some-wrapping of `Option<T>` default arguments will no longer re-wrap `Some(T)` or expressions evaluating to `None`. [#3461](https://github.com/PyO3/pyo3/pull/3461)
- Fix `IterNextOutput::Return` not returning a value on PyPy. [#3471](https://github.com/PyO3/pyo3/pull/3471)
- Emit compile errors instead of ignoring macro invocations inside `#[pymethods]` blocks. [#3491](https://github.com/PyO3/pyo3/pull/3491)
- Emit error on invalid arguments to `#[new]`, `#[classmethod]`, `#[staticmethod]`, and `#[classattr]`. [#3484](https://github.com/PyO3/pyo3/pull/3484)
- Disable `PyMarshal_WriteObjectToString` from `PyMarshal_ReadObjectFromString` with the `abi3` feature. [#3490](https://github.com/PyO3/pyo3/pull/3490)
- Fix FFI definitions for `_PyFrameEvalFunction` on Python 3.11 and up (it now receives a `_PyInterpreterFrame` opaque struct). [#3500](https://github.com/PyO3/pyo3/pull/3500)


## [0.19.2] - 2023-08-01

### Added

- Add FFI definitions `PyState_AddModule`, `PyState_RemoveModule` and `PyState_FindModule` for PyPy 3.9 and up. [#3295](https://github.com/PyO3/pyo3/pull/3295)
- Add FFI definitions `_PyObject_CallFunction_SizeT` and `_PyObject_CallMethod_SizeT`. [#3297](https://github.com/PyO3/pyo3/pull/3297)
- Add a "performance" section to the guide collecting performance-related tricks and problems. [#3304](https://github.com/PyO3/pyo3/pull/3304)
- Add `PyErr::Display` for all Python versions, and FFI symbol `PyErr_DisplayException` for Python 3.12. [#3334](https://github.com/PyO3/pyo3/pull/3334)
- Add FFI definition `PyType_GetDict()` for Python 3.12. [#3339](https://github.com/PyO3/pyo3/pull/3339)
- Add `PyAny::downcast_exact`. [#3346](https://github.com/PyO3/pyo3/pull/3346)
- Add `PySlice::full()` to construct a full slice (`::`). [#3353](https://github.com/PyO3/pyo3/pull/3353)

### Changed

- Update `PyErr` for 3.12 betas to avoid deprecated ffi methods. [#3306](https://github.com/PyO3/pyo3/pull/3306)
- Update FFI definitions of `object.h` for Python 3.12.0b4. [#3335](https://github.com/PyO3/pyo3/pull/3335)
- Update `pyo3::ffi` struct definitions to be compatible with 3.12.0b4. [#3342](https://github.com/PyO3/pyo3/pull/3342)
- Optimize conversion of `float` to `f64` (and `PyFloat::value`) on non-abi3 builds. [#3345](https://github.com/PyO3/pyo3/pull/3345)

### Fixed

- Fix timezone conversion bug for FixedOffset datetimes that were being incorrectly converted to and from UTC. [#3269](https://github.com/PyO3/pyo3/pull/3269)
- Fix `SystemError` raised in `PyUnicodeDecodeError_Create` on PyPy 3.10. [#3297](https://github.com/PyO3/pyo3/pull/3297)
- Correct FFI definition `Py_EnterRecursiveCall` to return `c_int` (was incorrectly returning `()`). [#3300](https://github.com/PyO3/pyo3/pull/3300)
- Fix case where `PyErr::matches` and `PyErr::is_instance` returned results inconsistent with `PyErr::get_type`. [#3313](https://github.com/PyO3/pyo3/pull/3313)
- Fix loss of panic message in `PanicException` when unwinding after the exception was "normalized". [#3326](https://github.com/PyO3/pyo3/pull/3326)
- Fix `PyErr::from_value` and `PyErr::into_value` losing traceback on conversion. [#3328](https://github.com/PyO3/pyo3/pull/3328)
- Fix reference counting of immortal objects on Python 3.12.0b4. [#3335](https://github.com/PyO3/pyo3/pull/3335)


## [0.19.1] - 2023-07-03

### Packaging

- Extend range of supported versions of `hashbrown` optional dependency to include version 0.14 [#3258](https://github.com/PyO3/pyo3/pull/3258)
- Extend range of supported versions of `indexmap` optional dependency to include version 2. [#3277](https://github.com/PyO3/pyo3/pull/3277)
- Support PyPy 3.10. [#3289](https://github.com/PyO3/pyo3/pull/3289)

### Added

- Add `pyo3::types::PyFrozenSetBuilder` to allow building a `PyFrozenSet` item by item. [#3156](https://github.com/PyO3/pyo3/pull/3156)
- Add support for converting to and from Python's `ipaddress.IPv4Address`/`ipaddress.IPv6Address` and `std::net::IpAddr`. [#3197](https://github.com/PyO3/pyo3/pull/3197)
- Add support for `num-bigint` feature in combination with `abi3`. [#3198](https://github.com/PyO3/pyo3/pull/3198)
- Add `PyErr_GetRaisedException()`, `PyErr_SetRaisedException()` to FFI definitions for Python 3.12 and later. [#3248](https://github.com/PyO3/pyo3/pull/3248)
- Add `Python::with_pool` which is a safer but more limited alternative to `Python::new_pool`. [#3263](https://github.com/PyO3/pyo3/pull/3263)
- Add `PyDict::get_item_with_error` on PyPy. [#3270](https://github.com/PyO3/pyo3/pull/3270)
- Allow `#[new]` methods may to return `Py<Self>` in order to return existing instances. [#3287](https://github.com/PyO3/pyo3/pull/3287)

### Fixed

- Fix conversion of classes implementing `__complex__` to `Complex` when using `abi3` or PyPy. [#3185](https://github.com/PyO3/pyo3/pull/3185)
- Stop suppressing unrelated exceptions in `PyAny::hasattr`. [#3271](https://github.com/PyO3/pyo3/pull/3271)
- Fix memory leak when creating `PySet` or `PyFrozenSet` or returning types converted into these internally, e.g. `HashSet` or `BTreeSet`. [#3286](https://github.com/PyO3/pyo3/pull/3286)


## [0.19.0] - 2023-05-31

### Packaging

- Correct dependency on syn to version 1.0.85 instead of the incorrect version 1.0.56. [#3152](https://github.com/PyO3/pyo3/pull/3152)

### Added

- Accept `text_signature` option (and automatically generate signature) for `#[new]` in `#[pymethods]`. [#2980](https://github.com/PyO3/pyo3/pull/2980)
- Add support for converting to and from Python's `decimal.Decimal` and `rust_decimal::Decimal`. [#3016](https://github.com/PyO3/pyo3/pull/3016)
- Add `#[pyo3(from_item_all)]` when deriving `FromPyObject` to specify `get_item` as getter for all fields. [#3120](https://github.com/PyO3/pyo3/pull/3120)
- Add `pyo3::exceptions::PyBaseExceptionGroup` for Python 3.11, and corresponding FFI definition `PyExc_BaseExceptionGroup`. [#3141](https://github.com/PyO3/pyo3/pull/3141)
- Accept `#[new]` with `#[classmethod]` to create a constructor which receives a (subtype's) class/`PyType` as its first argument. [#3157](https://github.com/PyO3/pyo3/pull/3157)
- Add `PyClass::get` and `Py::get` for GIL-indepedent access to classes with `#[pyclass(frozen)]`. [#3158](https://github.com/PyO3/pyo3/pull/3158)
- Add `PyAny::is_exact_instance` and `PyAny::is_exact_instance_of`. [#3161](https://github.com/PyO3/pyo3/pull/3161)

### Changed

- `PyAny::is_instance_of::<T>(obj)` is now equivalent to `T::is_type_of(obj)`, and now returns `bool` instead of `PyResult<bool>`. [#2881](https://github.com/PyO3/pyo3/pull/2881)
- Deprecate `text_signature` option on `#[pyclass]` structs. [#2980](https://github.com/PyO3/pyo3/pull/2980)
- No longer wrap `anyhow::Error`/`eyre::Report` containing a basic `PyErr` without a chain in a `PyRuntimeError`. [#3004](https://github.com/PyO3/pyo3/pull/3004)
- - Change `#[getter]` and `#[setter]` to use a common call "trampoline" to slightly reduce generated code size and compile times. [#3029](https://github.com/PyO3/pyo3/pull/3029)
- Improve default values for str, numbers and bool in automatically-generated `text_signature`. [#3050](https://github.com/PyO3/pyo3/pull/3050)
- Improve default value for `None` in automatically-generated `text_signature`. [#3066](https://github.com/PyO3/pyo3/pull/3066)
- Rename `PySequence::list` and `PySequence::tuple` to `PySequence::to_list` and `PySequence::to_tuple`. (The old names continue to exist as deprecated forms.) [#3111](https://github.com/PyO3/pyo3/pull/3111)
- Extend the lifetime of the GIL token returned by `PyRef::py` and `PyRefMut::py` to match the underlying borrow. [#3131](https://github.com/PyO3/pyo3/pull/3131)
- Safe access to the GIL, for example via `Python::with_gil`, is now locked inside of implementations of the `__traverse__` slot. [#3168](https://github.com/PyO3/pyo3/pull/3168)

### Removed

- Remove all functionality deprecated in PyO3 0.17, most prominently `Python::acquire_gil` is replaced by `Python::with_gil`. [#2981](https://github.com/PyO3/pyo3/pull/2981)

### Fixed

- Correct FFI definitions `PyGetSetDef`, `PyMemberDef`, `PyStructSequence_Field` and `PyStructSequence_Desc` to have `*const c_char` members for `name` and `doc` (not `*mut c_char`). [#3036](https://github.com/PyO3/pyo3/pull/3036)
- Fix panic on `fmt::Display`, instead return `"<unprintable object>"` string and report error via `sys.unraisablehook()` [#3062](https://github.com/PyO3/pyo3/pull/3062)
- Fix a compile error of "temporary value dropped while borrowed" when `#[pyfunction]`s take references into `#[pyclass]`es [#3142](https://github.com/PyO3/pyo3/pull/3142)
- Fix crashes caused by PyO3 applying deferred reference count updates when entering a `__traverse__` implementation. [#3168](https://github.com/PyO3/pyo3/pull/3168)
- Forbid running the `Drop` implementations of unsendable classes on other threads. [#3176](https://github.com/PyO3/pyo3/pull/3176)
- Fix a compile error when `#[pymethods]` items come from somewhere else (for example, as a macro argument) and a custom receiver like `Py<Self>` is used. [#3178](https://github.com/PyO3/pyo3/pull/3178)


## [0.18.3] - 2023-04-13

### Added

- Add `GILProtected<T>` to mediate concurrent access to a value using Python's global interpreter lock (GIL). [#2975](https://github.com/PyO3/pyo3/pull/2975)
- Support `PyASCIIObject` / `PyUnicode` and associated methods on big-endian architectures. [#3015](https://github.com/PyO3/pyo3/pull/3015)
- Add FFI definition `_PyDict_Contains_KnownHash()` for CPython 3.10 and up. [#3088](https://github.com/PyO3/pyo3/pull/3088)

### Fixed

- Fix compile error for `#[pymethods]` and `#[pyfunction]` called "output". [#3022](https://github.com/PyO3/pyo3/pull/3022)
- Fix compile error in generated code for magic methods implemented as a `#[staticmethod]`. [#3055](https://github.com/PyO3/pyo3/pull/3055)
- Fix `is_instance` for `PyDateTime` (would incorrectly check for a `PyDate`). [#3071](https://github.com/PyO3/pyo3/pull/3071)
- Fix upstream deprecation of `PyUnicode_InternImmortal` since Python 3.10. [#3071](https://github.com/PyO3/pyo3/pull/3087)


## [0.18.2] - 2023-03-24

### Packaging

- Disable default features of `chrono` to avoid depending on `time` v0.1.x. [#2939](https://github.com/PyO3/pyo3/pull/2939)

### Added

- Implement `IntoPy<PyObject>`, `ToPyObject` and `FromPyObject` for `Cow<[u8]>` to efficiently handle both `bytes` and `bytearray` objects. [#2899](https://github.com/PyO3/pyo3/pull/2899)
- Implement `IntoPy<PyObject>`, `ToPyObject` and `FromPyObject` for `Cell<T>`. [#3014](https://github.com/PyO3/pyo3/pull/3014)
- Add `PyList::to_tuple()`, as a convenient and efficient conversion from lists to tuples. [#3042](https://github.com/PyO3/pyo3/pull/3042)
- Add `PyTuple::to_list()`, as a convenient and efficient conversion from tuples to lists. [#3044](https://github.com/PyO3/pyo3/pull/3044)

### Changed

- Optimize `PySequence` conversion for `list` and `tuple` inputs. [#2944](https://github.com/PyO3/pyo3/pull/2944)
- Improve exception raised when creating `#[pyclass]` type object fails during module import. [#2947](https://github.com/PyO3/pyo3/pull/2947)
- Optimize `PyMapping` conversion for `dict` inputs. [#2954](https://github.com/PyO3/pyo3/pull/2954)
- Allow `create_exception!` to take a `dotted.module` to place the exception in a submodule. [#2979](https://github.com/PyO3/pyo3/pull/2979)

### Fixed

- Fix a reference counting race condition affecting `PyObject`s cloned in `allow_threads` blocks. [#2952](https://github.com/PyO3/pyo3/pull/2952)
- Fix `clippy::redundant_closure` lint on default arguments in `#[pyo3(signature = (...))]` annotations. [#2990](https://github.com/PyO3/pyo3/pull/2990)
- Fix `non_snake_case` lint on generated code in `#[pyfunction]` macro. [#2993](https://github.com/PyO3/pyo3/pull/2993)
- Fix some FFI definitions for the upcoming PyPy 3.10 release. [#3031](https://github.com/PyO3/pyo3/pull/3031)


## [0.18.1] - 2023-02-07

### Added

- Add `PyErr::write_unraisable()`. [#2889](https://github.com/PyO3/pyo3/pull/2889)
- Add `Python::Ellipsis()` and `PyAny::is_ellipsis()` methods. [#2911](https://github.com/PyO3/pyo3/pull/2911)
- Add `PyDict::update()` and `PyDict::update_if_missing()` methods. [#2912](https://github.com/PyO3/pyo3/pull/2912)

### Changed

- FFI definition `PyIter_Check` on CPython 3.7 is now implemented as `hasattr(type(obj), "__next__")`, which works correctly on all platforms and adds support for `abi3`. [#2914](https://github.com/PyO3/pyo3/pull/2914)
- Warn about unknown config keys in `PYO3_CONFIG_FILE` instead of denying. [#2926](https://github.com/PyO3/pyo3/pull/2926)

### Fixed

- Send errors returned by `__releasebuffer__` to `sys.unraisablehook` rather than causing `SystemError`. [#2886](https://github.com/PyO3/pyo3/pull/2886)
- Fix downcast to `PyIterator` succeeding for Python classes which did not implement `__next__`. [#2914](https://github.com/PyO3/pyo3/pull/2914)
- Fix segfault in `__traverse__` when visiting `None` fields of `Option<T: AsPyPointer>`. [#2921](https://github.com/PyO3/pyo3/pull/2921)
- Fix `#[pymethods(crate = "...")]` option being ignored. [#2923](https://github.com/PyO3/pyo3/pull/2923)
- Link against `pythonXY_d.dll` for debug Python builds on Windows. [#2937](https://github.com/PyO3/pyo3/pull/2937)


## [0.18.0] - 2023-01-17

### Packaging

- Relax `indexmap` optional depecency to allow `>= 1.6, < 2`. [#2849](https://github.com/PyO3/pyo3/pull/2849)
- Relax `hashbrown` optional dependency to allow `>= 0.9, < 0.14`. [#2875](https://github.com/PyO3/pyo3/pull/2875)
- Update `memoffset` dependency to 0.8. [#2875](https://github.com/PyO3/pyo3/pull/2875)

### Added

- Add `GILOnceCell::get_or_try_init` for fallible `GILOnceCell` initialization. [#2398](https://github.com/PyO3/pyo3/pull/2398)
- Add experimental feature `experimental-inspect` with `type_input()` and `type_output()` helpers to get the Python type of any Python-compatible object. [#2490](https://github.com/PyO3/pyo3/pull/2490) [#2882](https://github.com/PyO3/pyo3/pull/2882)
- The `#[pyclass]` macro can now take `get_all` and `set_all` to create getters and setters for every field. [#2692](https://github.com/PyO3/pyo3/pull/2692)
- Add `#[pyo3(signature = (...))]` option for `#[pyfunction]` and `#[pymethods]`. [#2702](https://github.com/PyO3/pyo3/pull/2702)
- `pyo3-build-config`: rebuild when `PYO3_ENVIRONMENT_SIGNATURE` environment variable value changes. [#2727](https://github.com/PyO3/pyo3/pull/2727)
- Add conversions between non-zero int types in `std::num` and Python `int`. [#2730](https://github.com/PyO3/pyo3/pull/2730)
- Add `Py::downcast()` as a companion to `PyAny::downcast()`, as well as `downcast_unchecked()` for both types. [#2734](https://github.com/PyO3/pyo3/pull/2734)
- Add types for all built-in `Warning` classes as well as `PyErr::warn_explicit`. [#2742](https://github.com/PyO3/pyo3/pull/2742)
- Add `abi3-py311` feature. [#2776](https://github.com/PyO3/pyo3/pull/2776)
- Add FFI definition `_PyErr_ChainExceptions()` for CPython. [#2788](https://github.com/PyO3/pyo3/pull/2788)
- Add FFI definitions `PyVectorcall_NARGS` and `PY_VECTORCALL_ARGUMENTS_OFFSET` for PyPy 3.8 and up. [#2811](https://github.com/PyO3/pyo3/pull/2811)
- Add `PyList::get_item_unchecked` for PyPy. [#2827](https://github.com/PyO3/pyo3/pull/2827)

### Changed

- PyO3's macros now emit a much nicer error message if function return values don't implement the required trait(s). [#2664](https://github.com/PyO3/pyo3/pull/2664)
- Use a TypeError, rather than a ValueError, when refusing to treat a str as a Vec. [#2685](https://github.com/PyO3/pyo3/pull/2685)
- Change `PyCFunction::new_closure` to take `name` and `doc` arguments. [#2686](https://github.com/PyO3/pyo3/pull/2686)
- `PyType::is_subclass`, `PyErr::is_instance` and `PyAny::is_instance` now take `&PyAny` instead of `&PyType` arguments, so that they work with objects that pretend to be types using `__subclasscheck__` and `__instancecheck__`. [#2695](https://github.com/PyO3/pyo3/pull/2695)
- Deprecate `#[args]` attribute and passing "args" specification directly to `#[pyfunction]` in favor of the new `#[pyo3(signature = (...))]` option. [#2702](https://github.com/PyO3/pyo3/pull/2702)
- Deprecate required arguments after `Option<T>` arguments to `#[pyfunction]` and `#[pymethods]` without also using `#[pyo3(signature)]` to specify whether the arguments should be required or have defaults. [#2703](https://github.com/PyO3/pyo3/pull/2703)
- Change `#[pyfunction]` and `#[pymethods]` to use a common call "trampoline" to slightly reduce generated code size and compile times. [#2705](https://github.com/PyO3/pyo3/pull/2705)
- `PyAny::cast_as()` and `Py::cast_as()` are now deprecated in favor of `PyAny::downcast()` and the new `Py::downcast()`. [#2734](https://github.com/PyO3/pyo3/pull/2734)
- Relax lifetime bounds on `PyAny::downcast()`. [#2734](https://github.com/PyO3/pyo3/pull/2734)
- Automatically generate `__text_signature__` for all Python functions created using `#[pyfunction]` and `#[pymethods]`. [#2784](https://github.com/PyO3/pyo3/pull/2784)
- Accept any iterator in `PySet::new` and `PyFrozenSet::new`. [#2795](https://github.com/PyO3/pyo3/pull/2795)
- Mixing `#[cfg(...)]` and `#[pyo3(...)]` attributes on `#[pyclass]` struct fields will now work. [#2796](https://github.com/PyO3/pyo3/pull/2796)
- Re-enable `PyFunction` on when building for abi3 or PyPy. [#2838](https://github.com/PyO3/pyo3/pull/2838)
- Improve `derive(FromPyObject)` to use `intern!` when applicable for `#[pyo3(item)]`. [#2879](https://github.com/PyO3/pyo3/pull/2879)

### Removed

- Remove the deprecated `pyproto` feature, `#[pyproto]` macro, and all accompanying APIs. [#2587](https://github.com/PyO3/pyo3/pull/2587)
- Remove all functionality deprecated in PyO3 0.16. [#2843](https://github.com/PyO3/pyo3/pull/2843)

### Fixed

- Disable `PyModule::filename` on PyPy. [#2715](https://github.com/PyO3/pyo3/pull/2715)
- `PyCodeObject` is now once again defined with fields on Python 3.7. [#2726](https://github.com/PyO3/pyo3/pull/2726)
- Raise a `TypeError` if `#[new]` pymethods with no arguments receive arguments when called from Python. [#2749](https://github.com/PyO3/pyo3/pull/2749)
- Use the `NOARGS` argument calling convention for methods that have a single `py: Python` argument (as a performance optimization). [#2760](https://github.com/PyO3/pyo3/pull/2760)
- Fix truncation of `isize` values to `c_long` in `PySlice::new`. [#2769](https://github.com/PyO3/pyo3/pull/2769)
- Fix soundness issue with FFI definition `PyUnicodeDecodeError_Create` on PyPy leading to indeterminate behavior (typically a `TypeError`). [#2772](https://github.com/PyO3/pyo3/pull/2772)
- Allow functions taking `**kwargs` to accept keyword arguments which share a name with a positional-only argument (as permitted by PEP 570). [#2800](https://github.com/PyO3/pyo3/pull/2800)
- Fix unresolved symbol for `PyObject_Vectorcall` on PyPy 3.9 and up. [#2811](https://github.com/PyO3/pyo3/pull/2811)
- Fix memory leak in `PyCFunction::new_closure`. [#2842](https://github.com/PyO3/pyo3/pull/2842)


## [0.17.3] - 2022-11-01

### Packaging

- Support Python 3.11. (Previous versions of PyO3 0.17 have been tested against Python 3.11 release candidates and are expected to be compatible, this is the first version tested against Python 3.11.0.) [#2708](https://github.com/PyO3/pyo3/pull/2708)

### Added

- Implemented `ExactSizeIterator` for `PyListIterator`, `PyDictIterator`, `PySetIterator` and `PyFrozenSetIterator`. [#2676](https://github.com/PyO3/pyo3/pull/2676)

### Fixed

- Fix regression of `impl FromPyObject for [T; N]` no longer accepting types passing `PySequence_Check`, e.g. NumPy arrays, since version 0.17.0. This the same fix that was applied `impl FromPyObject for Vec<T>` in version 0.17.1 extended to fixed-size arrays. [#2675](https://github.com/PyO3/pyo3/pull/2675)
- Fix UB in `FunctionDescription::extract_arguments_fastcall` due to creating slices from a null pointer. [#2687](https://github.com/PyO3/pyo3/pull/2687)


## [0.17.2] - 2022-10-04

### Packaging

- Added optional `chrono` feature to convert `chrono` types into types in the `datetime` module. [#2612](https://github.com/PyO3/pyo3/pull/2612)

### Added

- Add support for `num-bigint` feature on `PyPy`. [#2626](https://github.com/PyO3/pyo3/pull/2626)

### Fixed

- Correctly implement `__richcmp__` for enums, fixing `__ne__` returning always returning `True`. [#2622](https://github.com/PyO3/pyo3/pull/2622)
- Fix compile error since 0.17.0 with `Option<&SomePyClass>` argument with a default. [#2630](https://github.com/PyO3/pyo3/pull/2630)
- Fix regression of `impl FromPyObject for Vec<T>` no longer accepting types passing `PySequence_Check`, e.g. NumPy arrays, since 0.17.0. [#2631](https://github.com/PyO3/pyo3/pull/2631)

## [0.17.1] - 2022-08-28

### Fixed

- Fix visibility of `PyDictItems`, `PyDictKeys`, and `PyDictValues` types added in PyO3 0.17.0.
- Fix compile failure when using `#[pyo3(from_py_with = "...")]` attribute on an argument of type `Option<T>`. [#2592](https://github.com/PyO3/pyo3/pull/2592)
- Fix clippy `redundant-closure` lint on `**kwargs` arguments for `#[pyfunction]` and `#[pymethods]`. [#2595](https://github.com/PyO3/pyo3/pull/2595)

## [0.17.0] - 2022-08-23

### Packaging

- Update inventory dependency to `0.3` (the `multiple-pymethods` feature now requires Rust 1.62 for correctness). [#2492](https://github.com/PyO3/pyo3/pull/2492)

### Added

- Add `timezone_utc`. [#1588](https://github.com/PyO3/pyo3/pull/1588)
- Implement `ToPyObject` for `[T; N]`. [#2313](https://github.com/PyO3/pyo3/pull/2313)
- Add `PyDictKeys`, `PyDictValues` and `PyDictItems` Rust types. [#2358](https://github.com/PyO3/pyo3/pull/2358)
- Add `append_to_inittab`. [#2377](https://github.com/PyO3/pyo3/pull/2377)
- Add FFI definition `PyFrame_GetCode`. [#2406](https://github.com/PyO3/pyo3/pull/2406)
- Add `PyCode` and `PyFrame` high level objects. [#2408](https://github.com/PyO3/pyo3/pull/2408)
- Add FFI definitions `Py_fstring_input`, `sendfunc`, and `_PyErr_StackItem`. [#2423](https://github.com/PyO3/pyo3/pull/2423)
- Add `PyDateTime::new_with_fold`, `PyTime::new_with_fold`, `PyTime::get_fold`, and `PyDateTime::get_fold` for PyPy. [#2428](https://github.com/PyO3/pyo3/pull/2428)
- Add `#[pyclass(frozen)]`. [#2448](https://github.com/PyO3/pyo3/pull/2448)
- Accept `#[pyo3(name)]` on enum variants. [#2457](https://github.com/PyO3/pyo3/pull/2457)
- Add `CompareOp::matches` to implement `__richcmp__` as the result of a Rust `std::cmp::Ordering` comparison. [#2460](https://github.com/PyO3/pyo3/pull/2460)
- Add `PySuper` type. [#2486](https://github.com/PyO3/pyo3/pull/2486)
- Support PyPy on Windows with the `generate-import-lib` feature. [#2506](https://github.com/PyO3/pyo3/pull/2506)
- Add FFI definitions `Py_EnterRecursiveCall` and `Py_LeaveRecursiveCall`. [#2511](https://github.com/PyO3/pyo3/pull/2511)
- Add `PyDict::get_item_with_error`. [#2536](https://github.com/PyO3/pyo3/pull/2536)
- Add `#[pyclass(sequence)]` option. [#2567](https://github.com/PyO3/pyo3/pull/2567)

### Changed

- Change datetime constructors taking a `tzinfo` to take `Option<&PyTzInfo>` instead of `Option<&PyObject>`: `PyDateTime::new`, `PyDateTime::new_with_fold`, `PyTime::new`, and `PyTime::new_with_fold`. [#1588](https://github.com/PyO3/pyo3/pull/1588)
- Move `PyTypeObject::type_object` method to the `PyTypeInfo` trait, and deprecate the `PyTypeObject` trait. [#2287](https://github.com/PyO3/pyo3/pull/2287)
- Methods of `Py` and `PyAny` now accept `impl IntoPy<Py<PyString>>` rather than just `&str` to allow use of the `intern!` macro. [#2312](https://github.com/PyO3/pyo3/pull/2312)
- Change the deprecated `pyproto` feature to be opt-in instead of opt-out. [#2322](https://github.com/PyO3/pyo3/pull/2322)
- Emit better error messages when `#[pyfunction]` return types do not implement `IntoPy`. [#2326](https://github.com/PyO3/pyo3/pull/2326)
- Require `T: IntoPy` for `impl<T, const N: usize> IntoPy<PyObject> for [T; N]` instead of `T: ToPyObject`. [#2326](https://github.com/PyO3/pyo3/pull/2326)
- Deprecate the `ToBorrowedObject` trait. [#2333](https://github.com/PyO3/pyo3/pull/2333)
- Iterators over `PySet` and `PyDict` will now panic if the underlying collection is mutated during the iteration. [#2380](https://github.com/PyO3/pyo3/pull/2380)
- Iterators over `PySet` and `PyDict` will now panic if the underlying collection is mutated during the iteration. [#2380](https://github.com/PyO3/pyo3/pull/2380)
- Allow `#[classattr]` methods to be fallible. [#2385](https://github.com/PyO3/pyo3/pull/2385)
- Prevent multiple `#[pymethods]` with the same name for a single `#[pyclass]`. [#2399](https://github.com/PyO3/pyo3/pull/2399)
- Fixup `lib_name` when using `PYO3_CONFIG_FILE`. [#2404](https://github.com/PyO3/pyo3/pull/2404)
- Add a message to the `ValueError` raised by the `#[derive(FromPyObject)]` implementation for a tuple struct. [#2414](https://github.com/PyO3/pyo3/pull/2414)
- Allow `#[classattr]` methods to take `Python` argument. [#2456](https://github.com/PyO3/pyo3/pull/2456)
- Rework `PyCapsule` type to resolve soundness issues: [#2485](https://github.com/PyO3/pyo3/pull/2485)
  - `PyCapsule::new` and `PyCapsule::new_with_destructor` now take `name: Option<CString>` instead of `&CStr`.
  - The destructor `F` in `PyCapsule::new_with_destructor` must now be `Send`.
  - `PyCapsule::get_context` deprecated in favor of `PyCapsule::context` which doesn't take a `py: Python<'_>` argument.
  - `PyCapsule::set_context` no longer takes a `py: Python<'_>` argument.
  - `PyCapsule::name` now returns `PyResult<Option<&CStr>>` instead of `&CStr`.
- `FromPyObject::extract` for `Vec<T>` no longer accepts Python `str` inputs. [#2500](https://github.com/PyO3/pyo3/pull/2500)
- Ensure each `#[pymodule]` is only initialized once. [#2523](https://github.com/PyO3/pyo3/pull/2523)
- `pyo3_build_config::add_extension_module_link_args` now also emits linker arguments for `wasm32-unknown-emscripten`. [#2538](https://github.com/PyO3/pyo3/pull/2538)
- Type checks for `PySequence` and `PyMapping` now require inputs to inherit from (or register with) `collections.abc.Sequence` and `collections.abc.Mapping` respectively. [#2477](https://github.com/PyO3/pyo3/pull/2477)
- Disable `PyFunction` on when building for abi3 or PyPy. [#2542](https://github.com/PyO3/pyo3/pull/2542)
- Deprecate `Python::acquire_gil`. [#2549](https://github.com/PyO3/pyo3/pull/2549)

### Removed

- Remove all functionality deprecated in PyO3 0.15. [#2283](https://github.com/PyO3/pyo3/pull/2283)
- Make the `Dict`, `WeakRef` and `BaseNativeType` members of the `PyClass` private implementation details. [#2572](https://github.com/PyO3/pyo3/pull/2572)

### Fixed

- Enable incorrectly disabled FFI definition `PyThreadState_DeleteCurrent`. [#2357](https://github.com/PyO3/pyo3/pull/2357)
- Fix `wrap_pymodule` interactions with name resolution rules: it no longer "sees through" glob imports of `use submodule::*` when `submodule::submodule` is a `#[pymodule]`. [#2363](https://github.com/PyO3/pyo3/pull/2363)
- Correct FFI definition `PyEval_EvalCodeEx` to take `*const *mut PyObject` array arguments instead of `*mut *mut PyObject`. [#2368](https://github.com/PyO3/pyo3/pull/2368)
- Fix "raw-ident" structs (e.g. `#[pyclass] struct r#RawName`) incorrectly having `r#` at the start of the class name created in Python. [#2395](https://github.com/PyO3/pyo3/pull/2395)
- Correct FFI definition `Py_tracefunc` to be `unsafe extern "C" fn` (was previously safe). [#2407](https://github.com/PyO3/pyo3/pull/2407)
- Fix compile failure with `#[pyo3(from_py_with = "...")]` annotations on a field in a `#[derive(FromPyObject)]` struct. [#2414](https://github.com/PyO3/pyo3/pull/2414)
- Fix FFI definitions `_PyDateTime_BaseTime` and `_PyDateTime_BaseDateTime` lacking leading underscores in their names. [#2421](https://github.com/PyO3/pyo3/pull/2421)
- Remove FFI definition `PyArena` on Python 3.10 and up. [#2421](https://github.com/PyO3/pyo3/pull/2421)
- Fix FFI definition `PyCompilerFlags` missing member `cf_feature_version` on Python 3.8 and up. [#2423](https://github.com/PyO3/pyo3/pull/2423)
- Fix FFI definition `PyAsyncMethods` missing member `am_send` on Python 3.10 and up. [#2423](https://github.com/PyO3/pyo3/pull/2423)
- Fix FFI definition `PyGenObject` having multiple incorrect members on various Python versions. [#2423](https://github.com/PyO3/pyo3/pull/2423)
- Fix FFI definition `PySyntaxErrorObject` missing members `end_lineno` and `end_offset` on Python 3.10 and up. [#2423](https://github.com/PyO3/pyo3/pull/2423)
- Fix FFI definition `PyHeapTypeObject` missing member `ht_module` on Python 3.9 and up. [#2423](https://github.com/PyO3/pyo3/pull/2423)
- Fix FFI definition `PyFrameObject` having multiple incorrect members on various Python versions. [#2424](https://github.com/PyO3/pyo3/pull/2424) [#2434](https://github.com/PyO3/pyo3/pull/2434)
- Fix FFI definition `PyTypeObject` missing deprecated field `tp_print` on Python 3.8. [#2428](https://github.com/PyO3/pyo3/pull/2428)
- Fix FFI definitions `PyDateTime_CAPI`. `PyDateTime_Date`, `PyASCIIObject`, `PyBaseExceptionObject`, `PyListObject`, and `PyTypeObject` on PyPy. [#2428](https://github.com/PyO3/pyo3/pull/2428)
- Fix FFI definition `_inittab` field `initfunc` typo'd as `initfun`. [#2431](https://github.com/PyO3/pyo3/pull/2431)
- Fix FFI definitions `_PyDateTime_BaseTime` and `_PyDateTime_BaseDateTime` incorrectly having `fold` member. [#2432](https://github.com/PyO3/pyo3/pull/2432)
- Fix FFI definitions `PyTypeObject`. `PyHeapTypeObject`, and `PyCFunctionObject` having incorrect members on PyPy 3.9. [#2433](https://github.com/PyO3/pyo3/pull/2433)
- Fix FFI definition `PyGetSetDef` to have `*const c_char` for `doc` member (not `*mut c_char`). [#2439](https://github.com/PyO3/pyo3/pull/2439)
- Fix `#[pyo3(from_py_with = "...")]` being ignored for 1-element tuple structs and transparent structs. [#2440](https://github.com/PyO3/pyo3/pull/2440)
- Use `memoffset` to avoid UB when computing `PyCell` layout. [#2450](https://github.com/PyO3/pyo3/pull/2450)
- Fix incorrect enum names being returned by the generated `repr` for enums renamed by `#[pyclass(name = "...")]` [#2457](https://github.com/PyO3/pyo3/pull/2457)
- Fix `PyObject_CallNoArgs` incorrectly being available when building for abi3 on Python 3.9. [#2476](https://github.com/PyO3/pyo3/pull/2476)
- Fix several clippy warnings generated by `#[pyfunction]` arguments. [#2503](https://github.com/PyO3/pyo3/pull/2503)

## [0.16.6] - 2022-08-23

### Changed

- Fix soundness issues with `PyCapsule` type with select workarounds. Users are encourage to upgrade to PyO3 0.17 at their earliest convenience which contains API breakages which fix the issues in a long-term fashion. [#2522](https://github.com/PyO3/pyo3/pull/2522)
  - `PyCapsule::new` and `PyCapsule::new_with_destructor` now take ownership of a copy of the `name` to resolve a possible use-after-free.
  - `PyCapsule::name` now returns an empty `CStr` instead of dereferencing a null pointer if the capsule has no name.
  - The destructor `F` in `PyCapsule::new_with_destructor` will never be called if the capsule is deleted from a thread other than the one which the capsule was created in (a warning will be emitted).
- Panics during drop of panic payload caught by PyO3 will now abort. [#2544](https://github.com/PyO3/pyo3/pull/2544)

## [0.16.5] - 2022-05-15

### Added

- Add an experimental `generate-import-lib` feature to support auto-generating non-abi3 python import libraries for Windows targets. [#2364](https://github.com/PyO3/pyo3/pull/2364)
- Add FFI definition `Py_ExitStatusException`. [#2374](https://github.com/PyO3/pyo3/pull/2374)

### Changed

- Deprecate experimental `generate-abi3-import-lib` feature in favor of the new `generate-import-lib` feature. [#2364](https://github.com/PyO3/pyo3/pull/2364)

### Fixed

- Added missing `warn_default_encoding` field to `PyConfig` on 3.10+. The previously missing field could result in incorrect behavior or crashes. [#2370](https://github.com/PyO3/pyo3/pull/2370)
- Fixed order of `pathconfig_warnings` and `program_name` fields of `PyConfig` on 3.10+. Previously, the order of the fields was swapped and this could lead to incorrect behavior or crashes. [#2370](https://github.com/PyO3/pyo3/pull/2370)

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
- Deprecate `pyo3_build_config::cross_compiling` in favor of `pyo3_build_config::cross_compiling_from_to`. [#2253](https://github.com/PyO3/pyo3/pull/2253)

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
- `PyDateTimeAPI` and `PyDateTime_TimeZone_UTC` are now unsafe functions instead of statics. [#2126](https://github.com/PyO3/pyo3/pull/2126)
- `PyDateTimeAPI` does not implicitly call `PyDateTime_IMPORT` anymore to reflect the original Python API more closely. Before the first call to `PyDateTime_IMPORT` a null pointer is returned. Therefore before calling any of the following FFI functions `PyDateTime_IMPORT` must be called to avoid undefined behavior: [#2126](https://github.com/PyO3/pyo3/pull/2126)
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
- Remove `PartialEq` impl for `Py` and `PyAny` (use the new `is` instead). [#2183](https://github.com/PyO3/pyo3/pull/2183)

### Fixed

- Fix undefined symbol for `PyObject_HasAttr` on PyPy. [#2025](https://github.com/PyO3/pyo3/pull/2025)
- Fix memory leak in `PyErr::into_value`. [#2026](https://github.com/PyO3/pyo3/pull/2026)
- Fix clippy warning `needless-option-as-deref` in code generated by `#[pyfunction]` and `#[pymethods]`. [#2040](https://github.com/PyO3/pyo3/pull/2040)
- Fix undefined behavior in `PySlice::indices`. [#2061](https://github.com/PyO3/pyo3/pull/2061)
- Fix the `wrap_pymodule!` macro using the wrong name for a `#[pymodule]` with a `#[pyo3(name = "..")]` attribute. [#2081](https://github.com/PyO3/pyo3/pull/2081)
- Fix magic methods in `#[pymethods]` accepting implementations with the wrong number of arguments. [#2083](https://github.com/PyO3/pyo3/pull/2083)
- Fix panic in `#[pyfunction]` generated code when a required argument following an `Option` was not provided.  [#2093](https://github.com/PyO3/pyo3/pull/2093)
- Fixed undefined behavior caused by incorrect `ExactSizeIterator` implementations. [#2124](https://github.com/PyO3/pyo3/pull/2124)
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

- Add implementations for `Py::as_ref` and `Py::into_ref` for `Py<PySequence>`, `Py<PyIterator>` and `Py<PyMapping>`. [#1682](https://github.com/PyO3/pyo3/pull/1682)
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
- `PYO3_CROSS_LIB_DIR` environment variable no long required when compiling for x86-64 Python from macOS arm64 and reverse. [#1428](https://github.com/PyO3/pyo3/pull/1428)
- Fix FFI definition `_PyEval_RequestCodeExtraIndex`, which took an argument of the wrong type. [#1429](https://github.com/PyO3/pyo3/pull/1429)
- Fix FFI definition `PyIndex_Check` missing with the `abi3` feature. [#1436](https://github.com/PyO3/pyo3/pull/1436)
- Fix incorrect `TypeError` raised when keyword-only argument passed along with a positional argument in `*args`. [#1440](https://github.com/PyO3/pyo3/pull/1440)
- Fix inability to use a named lifetime for `&PyTuple` of `*args` in `#[pyfunction]`. [#1440](https://github.com/PyO3/pyo3/pull/1440)
- Fix use of Python argument for `#[pymethods]` inside macro expansions. [#1505](https://github.com/PyO3/pyo3/pull/1505)
- No longer include `__doc__` in `__all__` generated for `#[pymodule]`. [#1509](https://github.com/PyO3/pyo3/pull/1509)
- Always use cross-compiling configuration if any of the `PYO3_CROSS` family of environment variables are set. [#1514](https://github.com/PyO3/pyo3/pull/1514)
- Support `EnvironmentError`, `IOError`, and `WindowsError` on PyPy. [#1533](https://github.com/PyO3/pyo3/pull/1533)
- Fix unnecessary rebuilds when cycling between `cargo check` and `cargo clippy` in a Python virtualenv. [#1557](https://github.com/PyO3/pyo3/pull/1557)
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
- Deprecate FFI definitions for `PyUnicode_FromUnicode`, `PyUnicode_AsUnicode` and `PyUnicode_AsUnicodeAndSize`, which will be removed from 3.12 and up due to [PEP 623](https://www.python.org/dev/peps/pep-0623/). [#1370](https://github.com/PyO3/pyo3/pull/1370)

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

- Add support for building for CPython limited API. Opting-in to the limited API enables a single extension wheel built with PyO3 to be installable on multiple Python versions. This required a few minor changes to runtime behavior of of PyO3 `#[pyclass]` types. See the migration guide for full details. [#1152](https://github.com/PyO3/pyo3/pull/1152)
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
- `"*"` in a `#[pyfunction()]` argument list incorrectly accepting any number of positional arguments (use `args = "*"` when this behavior is desired). #[792](https://github.com/PyO3/pyo3/pull/792)
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
- All exceptions are constructed with `py_err` instead of `new`, as they return `PyErr` and not `Self`.
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

[Unreleased]: https://github.com/pyo3/pyo3/compare/v0.26.0...HEAD
[0.26.0]: https://github.com/pyo3/pyo3/compare/v0.25.1...v0.26.0
[0.25.1]: https://github.com/pyo3/pyo3/compare/v0.25.0...v0.25.1
[0.25.0]: https://github.com/pyo3/pyo3/compare/v0.24.2...v0.25.0
[0.24.2]: https://github.com/pyo3/pyo3/compare/v0.24.1...v0.24.2
[0.24.1]: https://github.com/pyo3/pyo3/compare/v0.24.0...v0.24.1
[0.24.0]: https://github.com/pyo3/pyo3/compare/v0.23.5...v0.24.0
[0.23.5]: https://github.com/pyo3/pyo3/compare/v0.23.4...v0.23.5
[0.23.4]: https://github.com/pyo3/pyo3/compare/v0.23.3...v0.23.4
[0.23.3]: https://github.com/pyo3/pyo3/compare/v0.23.2...v0.23.3
[0.23.2]: https://github.com/pyo3/pyo3/compare/v0.23.1...v0.23.2
[0.23.1]: https://github.com/pyo3/pyo3/compare/v0.23.0...v0.23.1
[0.23.0]: https://github.com/pyo3/pyo3/compare/v0.22.5...v0.23.0
[0.22.5]: https://github.com/pyo3/pyo3/compare/v0.22.4...v0.22.5
[0.22.4]: https://github.com/pyo3/pyo3/compare/v0.22.3...v0.22.4
[0.22.3]: https://github.com/pyo3/pyo3/compare/v0.22.2...v0.22.3
[0.22.2]: https://github.com/pyo3/pyo3/compare/v0.22.1...v0.22.2
[0.22.1]: https://github.com/pyo3/pyo3/compare/v0.22.0...v0.22.1
[0.22.0]: https://github.com/pyo3/pyo3/compare/v0.21.2...v0.22.0
[0.21.2]: https://github.com/pyo3/pyo3/compare/v0.21.1...v0.21.2
[0.21.1]: https://github.com/pyo3/pyo3/compare/v0.21.0...v0.21.1
[0.21.0]: https://github.com/pyo3/pyo3/compare/v0.20.3...v0.21.0
[0.21.0-beta.0]: https://github.com/pyo3/pyo3/compare/v0.20.3...v0.21.0-beta.0
[0.20.3]: https://github.com/pyo3/pyo3/compare/v0.20.2...v0.20.3
[0.20.2]: https://github.com/pyo3/pyo3/compare/v0.20.1...v0.20.2
[0.20.1]: https://github.com/pyo3/pyo3/compare/v0.20.0...v0.20.1
[0.20.0]: https://github.com/pyo3/pyo3/compare/v0.19.2...v0.20.0
[0.19.2]: https://github.com/pyo3/pyo3/compare/v0.19.1...v0.19.2
[0.19.1]: https://github.com/pyo3/pyo3/compare/v0.19.0...v0.19.1
[0.19.0]: https://github.com/pyo3/pyo3/compare/v0.18.3...v0.19.0
[0.18.3]: https://github.com/pyo3/pyo3/compare/v0.18.2...v0.18.3
[0.18.2]: https://github.com/pyo3/pyo3/compare/v0.18.1...v0.18.2
[0.18.1]: https://github.com/pyo3/pyo3/compare/v0.18.0...v0.18.1
[0.18.0]: https://github.com/pyo3/pyo3/compare/v0.17.3...v0.18.0
[0.17.3]: https://github.com/pyo3/pyo3/compare/v0.17.2...v0.17.3
[0.17.2]: https://github.com/pyo3/pyo3/compare/v0.17.1...v0.17.2
[0.17.1]: https://github.com/pyo3/pyo3/compare/v0.17.0...v0.17.1
[0.17.0]: https://github.com/pyo3/pyo3/compare/v0.16.6...v0.17.0
[0.16.6]: https://github.com/pyo3/pyo3/compare/v0.16.5...v0.16.6
[0.16.5]: https://github.com/pyo3/pyo3/compare/v0.16.4...v0.16.5
[0.16.4]: https://github.com/pyo3/pyo3/compare/v0.16.3...v0.16.4
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
[0.8.5]: https://github.com/pyo3/pyo3/compare/v0.8.4...v0.8.5
[0.8.4]: https://github.com/pyo3/pyo3/compare/v0.8.3...v0.8.4
[0.8.3]: https://github.com/pyo3/pyo3/compare/v0.8.2...v0.8.3
[0.8.2]: https://github.com/pyo3/pyo3/compare/v0.8.1...v0.8.2
[0.8.1]: https://github.com/pyo3/pyo3/compare/v0.8.0...v0.8.1
[0.8.0]: https://github.com/pyo3/pyo3/compare/v0.7.0...v0.8.0
[0.7.0]: https://github.com/pyo3/pyo3/compare/v0.6.0...v0.7.0
[0.6.0]: https://github.com/pyo3/pyo3/compare/v0.5.3...v0.6.0
[0.5.3]: https://github.com/pyo3/pyo3/compare/v0.5.2...v0.5.3
[0.5.2]: https://github.com/pyo3/pyo3/compare/v0.5.1...v0.5.2
[0.5.1]: https://github.com/pyo3/pyo3/compare/v0.5.0...v0.5.1
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
