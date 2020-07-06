# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](http://keepachangelog.com/en/1.0.0/)
and this project adheres to [Semantic Versioning](http://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.11.1] - 2020-06-30
### Added
- `#[pyclass(unsendable)]`. [#1009](https://github.com/PyO3/pyo3/pull/1009)

### Changed
- Update `parking_lot` dependency to `0.11`. [#1010](https://github.com/PyO3/pyo3/pull/1010)
- `PyString::to_string` is now renamed `to_str` and returns `&str` instead of `Cow<str>`. [#1023](https://github.com/PyO3/pyo3/pull/1023)

### Removed
- Remove `PyString::as_bytes`. [#1023](https://github.com/PyO3/pyo3/pull/1023)
- Remove `Python::register_gil`. [#1023](https://github.com/PyO3/pyo3/pull/1023)

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
- Use existing fields and methods before calling custom __getattr__. [#505](https://github.com/PyO3/pyo3/pull/512)
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

## 0.1.0 - 07-23-2017
### Added
- Initial release

[Unreleased]: https://github.com/pyo3/pyo3/compare/v0.11.1...HEAD
[0.11.0] https://github.com/pyo3/pyo3/compare/v0.11.0...v0.11.1
[0.11.0] https://github.com/pyo3/pyo3/compare/v0.10.1...v0.11.0
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
