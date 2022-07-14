# Changelog


All notable changes to this project will be documented in this file. For help with updating to new
PyO3 versions, please see the [migration guide](https://pyo3.rs/latest/migration.html).

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/)
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.16.6](https://github.com/pyo3/pyo3/compare/v0.16.4...v0.16.6) - 2022-07-14

### Added

- Add regression test for PEP 587

- Add `CompareOp::matches` ([#2460](https://github.com/PyO3/pyo3/pull/2460))



### Fixed

- Fix some typos

Signed-off-by: cuishuang <imcusg@gmail.com>

- Fixed comment in string.rs
- Fixed name of initfunc field on _inittab

## [0.16.2](https://github.com/pyo3/pyo3/compare/v0.16.1...v0.16.2) - 2022-03-15

### Fixed

- Fixed formatting in changelog ([#2218](https://github.com/PyO3/pyo3/pull/2218))



## [0.16.1](https://github.com/pyo3/pyo3/compare/v0.16.0...v0.16.1) - 2022-03-05

### Added

- Add changelog for 2198

- Added missing proc-macro invocation to migration guide ([#2209](https://github.com/PyO3/pyo3/pull/2209))



## [0.16.0](https://github.com/pyo3/pyo3/compare/v0.15.1...v0.16.0) - 2022-02-27

### Added

- Add PyType::is_subclass_of and PyAny::is_instance_of

which get the type to check against as an arguments,
as opposed to a compile-time generic type.

- Add maturin to requirements-dev files

- Add pyheck to examples

### Fixed

- Fix link

- Fix tests

- Fix guide

- Remove excess argument for `_PyCFunctionFast`
- Guide example for pyclass expansion
- Memory leak in Option<T>::as_ptr

## [0.15.1](https://github.com/pyo3/pyo3/compare/v0.15.0...v0.15.1) - 2021-11-18

### Added

- Add doc_auto_cfg feature

- Add missing annotation to PyCounter.__call__

This patch annotates the `__call__` method of `PyCounter` in example: callable
objects with `#[args(args="*", kwargs="**")]`. Without it (at least in PyO3
0.15.0) the example fails with `TypeError: counter.__call__() missing 1
required positional argument: 'args'`.


## [0.15.0](https://github.com/pyo3/pyo3/compare/v0.14.2...v0.15.0) - 2021-11-03

### Added

- Add decorator example

- Add two missing feature restrictions for nightly

Found with `cargo hack --feature-powerset test`.


### Fixed

- Fix extends param

- Fixup! ffi: cleanup pythonrun

- Fix broken link


## [0.14.2](https://github.com/pyo3/pyo3/compare/v0.14.1...v0.14.2) - 2021-08-09

### Added

- Add CHANGELOG.md


### Fixed

- Fix compiler warning

Fix issue [link](https://github.com/rust-lang/rust/issues/79813) which
will eventually deny trailing semicolons in expression macro in rust compiler.


## [0.14.1](https://github.com/pyo3/pyo3/compare/v0.14.0...v0.14.1) - 2021-07-04

### Fixed

- Fix typo

Co-authored-by: Georg Brandl <georg@python.org>
- Fix punctuation

Co-authored-by: Georg Brandl <georg@python.org>
- Fix punctuation

Co-authored-by: Georg Brandl <georg@python.org>

## [0.14.0](https://github.com/pyo3/pyo3/compare/v0.13.2...v0.14.0) - 2021-07-03

### Added

- Add #[pyo3(from_py_with="...")] attribute ([#1411](https://github.com/PyO3/pyo3/pull/1411))

* allow from_py_with inside #[derive(FromPyObject)]
* split up FnSpec::parse
- Added some docsrs tags

- Added suggested changes

- Added doc alias for intopy

- Added package.metadata.docs.rs information

- Added suggested changes

- Add renaming method to guide

- Add faq doctest

- Add num_bigint docs

- Add documentation


### Fixed

- Fixed tabs in doc comment

- Fixed doc typo

- Fixed extra whitespace

- Fixed whitespace

- Fixed dead module link on pypy


## [0.13.2](https://github.com/pyo3/pyo3/compare/v0.13.1...v0.13.2) - 2021-02-12

### Added

- Add some safety notes


## [0.13.1](https://github.com/pyo3/pyo3/compare/v0.13.0...v0.13.1) - 2021-01-10

### Fixed

- Fix pypy compilation

- Fix pypy non-limited build

- Fix pypy3 build

- Fix errors in PR


## [0.12.1](https://github.com/pyo3/pyo3/compare/v0.12.0...v0.12.1) - 2020-09-16

### Fixed

- Fixed markdown syntax

## [0.12.0](https://github.com/pyo3/pyo3/compare/v0.11.1...v0.12.0) - 2020-09-12

### Fixed

- Fix complexity of finding and parsing

- Fix changelog and added comment for finding sysconfigdata

- Fixes


## [0.11.0](https://github.com/pyo3/pyo3/compare/v0.10.1...v0.11.0) - 2020-06-28

### Fixed

- Fix api comment


## [0.9.0-alpha.1](https://github.com/pyo3/pyo3/compare/v0.8.5...v0.9.0-alpha.1) - 2020-01-17

### Added

- Added PyModule.add_class to guide and better explanation of arguments


### Fixed

- Fix code validation test


## [0.8.4](https://github.com/pyo3/pyo3/compare/v0.8.3...v0.8.4) - 2019-12-14

### Added

- Add text_signature to documentation

- Add text_signature to changelog


### Fixed

- Fix compile error


## [0.8.0](https://github.com/pyo3/pyo3/compare/v0.7.0...v0.8.0) - 2019-09-05

### Added

- Adding a small testcase for rhs arithmetics


### Fixed

- Fixup


## [0.6.0](https://github.com/pyo3/pyo3/compare/v0.6.0-alpha.4...v0.6.0) - 2019-03-28

### Fixed

- Fix #398


## [0.6.0-alpha.2](https://github.com/pyo3/pyo3/compare/v0.6.0-alpha.1...v0.6.0-alpha.2) - 2019-02-01

### Added

- Add documentation on cross compiling

- Add doc for C #define regex


## [0.2.5](https://github.com/pyo3/pyo3/compare/v0.2.4...v0.2.5) - 2018-02-21

### Fixed

- Fix python3.7 support


## [0.2.4](https://github.com/pyo3/pyo3/compare/v0.2.3...v0.2.4) - 2018-01-19

### Added

- Add link to crate

- Add a rust-toolchain file in the repo to indicate the toolchain used


### Fixed

- Fix travis build


## [0.2.3](https://github.com/pyo3/pyo3/compare/...v0.2.3) - 2017-11-27

### Added

- Added example of implementing a basic python module

- Add PyAsyncMethods support

- Add class properties

- Add PyErr to io::Error convert

- Add PySlice

- Add PyByteArray

- Added convinience method

- Add generator methods

- Added coro and asyncgen types

- Added ToPyObject for ()

- Add license

- Add PyAsyncProtocol

- Add mapping protocol

- Added sequence protocol

- Add number protocol

- Added properties support

- Added descriptor protocol support

- Added fn spec for args parsing

- Add agr extraction for setters, allow non snake vars

- Added PyIterProtocol

- Add __new__ and __call__ support

- Add description

- Added helper method Python::init, shortcut for Py::new(..)

- Added pptr pointer

- Add mirror ptr types for native py classes

- Add ptr support to class

- Add class and static methods #17

- Add test for custom class name

- Add free list support

- Added PyInt for py2

- Added proper PyString for py2 handling

- Add class doc string

- Added __doc__ to methods

- Add pyfn to python2

- Added __unicode__ method to PyObjectProtocol

- Add clippy and doc to travis

- Add specialization to docs

- Add downcast traits to Py

- Add release pool

- Added doc strings

- Add some docs

- Add basic args test

- Added try downcast methods to PyDowncastFrom trait

- Added PyTuple::slice and PyTuple::split_from methods

- Added PyList::append method, added refcnt tests

- Add PyDict keys,values,items methods

- Add version

- Add weakref support #56

- Add appveyor support

- Add base support for inheritance

- Add guid entry for base class

- Added PySuper and __init__ support

- Add prelude mod

- Add std TryFrom impl for type conversion #73

- Added IntoIterator for PyList

- Add IntoIterator for PyDict

- Add IntoPyDictPointer impl for tuple

- Add convenience method to PyErr and exception types

- Add convenience call related methods to ObjectProtocol

- Added use; restore exception

- Add badges

- Add badges

### Fixed

- Fix issues in bb13ec, support utf16, python fom PATH

* fix ucs4 build broken by bb13ec
* add utf16 decoding to unicode.from_py_object for
  narrow unicode builds
* change unicode narrow/wide cfg flag to be
  Py_UNICODE_SIZE_4 not Py_UNICODE_WIDE, which doesn't
  appear in sysconfig
* support framework builds on os x
* python27-sys exports compilation flags as cargo vars,
  and rust-python resurrects them as cfg flags
* travis runs against local python27-sys
* rust-cpython depends on git python27-sys, because
  the one on cargo is now incompatible with it (since bb13ec)

- Fix __buffer_get__ return type

- Fix optional arg generation

- Fix generated type name

- Fixed gil lifetime; pass py object into protocol methods

- Fix pyclass methods

- Fix sequence protocol; convert buffer protocol

- Fix ptr borrowing PyTuple::get_item

- Fix py2 travis build

- Fix python exptension export

- Fix missing use

- Fix readme

- Fix doc command

- Fix doc links

- Fix module export functions

- Fix env check

- Fix unchecked downcast; added objects pool

- Fix kwargs for py methods

- Fix doc tests; fix unsued import warning

- Fix imports

- Fix python2 extension module initialization

- Fix doc comments

- Fix class,md

- Fix api inconsystency

- Fix error from instancel fixed bool

- Fix ref counter for get_kwargs

- Fix pointer release list

- Fix PyNumberProtocol methods #48

- Fix mod name

- Fix memory leak

- Fix memory leak in call and call_method

- Fix fn names

- Fix python2 related code

- Fix memory leak in PyList::set_item and insert_item

- Fix modul init fn for python2

- Fix name

- Fix license name

- Fix doc link

- Fix readme

- Fix long type

- Fix unstable tests

- Fix clippy warnings

- Fix import_exception macro

- Fix doc links

- Fix #[prop] impl

- Fix travis


### Modified

- Modify PyDict


<!-- generated by git-cliff -->
