//! Raw FFI declarations for Python's C API.
//!
//! PyO3 can be used to write native Python modules or run Python code and modules from Rust.
//!
//! This crate just provides low level bindings to the Python interpreter.
//! It is meant for advanced users only - regular PyO3 users shouldn't
//! need to interact with this crate at all.
//!
//! The contents of this crate are not documented here, as it would entail
//! basically copying the documentation from CPython. Consult the [Python/C API Reference
//! Manual][capi] for up-to-date documentation.
//!
//! # Safety
//!
//! The functions in this crate lack individual safety documentation, but
//! generally the following apply:
//! - Pointer arguments have to point to a valid Python object of the correct type,
//! although null pointers are sometimes valid input.
//! - The vast majority can only be used safely while the GIL is held.
//! - Some functions have additional safety requirements, consult the
//! [Python/C API Reference Manual][capi]
//! for more information.
//!
//!
//! # Feature flags
//!
//! PyO3 uses [feature flags] to enable you to opt-in to additional functionality. For a detailed
//! description, see the [Features chapter of the guide].
//!
//! ## Optional feature flags
//!
//! The following features customize PyO3's behavior:
//!
//! - `abi3`: Restricts PyO3's API to a subset of the full Python API which is guaranteed by
//! [PEP 384] to be forward-compatible with future Python versions.
//! - `extension-module`: This will tell the linker to keep the Python symbols unresolved, so that
//! your module can also be used with statically linked Python interpreters. Use this feature when
//! building an extension module.
//!
//! ## `rustc` environment flags
//!
//! PyO3 uses `rustc`'s `--cfg` flags to enable or disable code used for different Python versions.
//! If you want to do this for your own crate, you can do so with the [`pyo3-build-config`] crate.
//!
//! - `Py_3_7`, `Py_3_8`, `Py_3_9`, `Py_3_10`: Marks code that is only enabled when
//!  compiling for a given minimum Python version.
//! - `Py_LIMITED_API`: Marks code enabled when the `abi3` feature flag is enabled.
//! - `PyPy` - Marks code enabled when compiling for PyPy.
//!
//! # Minimum supported Rust and Python versions
//!
//! PyO3 supports the following software versions:
//!   - Python 3.7 and up (CPython and PyPy)
//!   - Rust 1.48 and up
//!
//! # Example: Building Python Native modules
//!
//! PyO3 can be used to generate a native Python module. The easiest way to try this out for the
//! first time is to use [`maturin`]. `maturin` is a tool for building and publishing Rust-based
//! Python packages with minimal configuration. The following steps set up some files for an example
//! Python module, install `maturin`, and then show how to build and import the Python module.
//!
//! First, create a new folder (let's call it `string_sum`) containing the following two files:
//!
//! **`Cargo.toml`**
//!
//! ```toml
//! [lib]
//! name = "string_sum"
//! # "cdylib" is necessary to produce a shared library for Python to import from.
//! #
//! # Downstream Rust code (including code in `bin/`, `examples/`, and `tests/`) will not be able
//! # to `use string_sum;` unless the "rlib" or "lib" crate type is also included, e.g.:
//! # crate-type = ["cdylib", "rlib"]
//! crate-type = ["cdylib"]
//!
//! [dependencies.pyo3-ffi]
// workaround for `extended_key_value_attributes`: https://github.com/rust-lang/rust/issues/82768#issuecomment-803935643
#![cfg_attr(docsrs, cfg_attr(docsrs, doc = concat!("version = \"", env!("CARGO_PKG_VERSION"),  "\"")))]
#![cfg_attr(not(docsrs), doc = "version = \"*\"")]
//! features = ["extension-module"]
//! ```
//!
//! **`src/lib.rs`**
//! ```rust
//! use std::os::raw::c_char;
//! use std::ptr;
//!
//! use pyo3_ffi::*;
//!
//! #[allow(non_snake_case)]
//! #[no_mangle]
//! pub unsafe extern "C" fn PyInit_string_sum() -> *mut PyObject {
//!     let init = PyModuleDef {
//!         m_base: PyModuleDef_HEAD_INIT,
//!         m_name: "string_sum\0".as_ptr() as *const c_char,
//!         m_doc: std::ptr::null(),
//!         m_size: 0,
//!         m_methods: std::ptr::null_mut(),
//!         m_slots: std::ptr::null_mut(),
//!         m_traverse: None,
//!         m_clear: None,
//!         m_free: None,
//!     };
//!
//!     let mptr = PyModule_Create(Box::into_raw(Box::new(init)));
//!     let version = env!("CARGO_PKG_VERSION");
//!     PyModule_AddObject(
//!         mptr,
//!         "__version__\0".as_ptr() as *const c_char,
//!         PyUnicode_FromStringAndSize(version.as_ptr() as *const c_char, version.len() as isize),
//!     );
//!
//!     let wrapped_sum_as_string = PyMethodDef {
//!         ml_name: "sum_as_string\0".as_ptr() as *const c_char,
//!         ml_meth: MlMeth {
//!             _PyCFunctionFast: Some(sum_as_string)
//!         },
//!         ml_flags: METH_FASTCALL,
//!         ml_doc: "returns the sum of two integers as a string\0".as_ptr() as *const c_char,
//!     };
//!
//!     // PyModule_AddObject can technically fail.
//!     // For more involved applications error checking may be necessary
//!     PyModule_AddObject(
//!         mptr,
//!         "sum_as_string\0".as_ptr() as *const c_char,
//!         PyCFunction_NewEx(
//!             Box::into_raw(Box::new(wrapped_sum_as_string)),
//!             std::ptr::null_mut(),
//!             PyUnicode_InternFromString("string_sum\0".as_ptr() as *const c_char),
//!         ),
//!     );
//!
//!     let all = ["__all__\0", "__version__\0", "sum_as_string\0"];
//!
//!     let pyall = PyTuple_New(all.len() as isize);
//!     for (i, obj) in all.iter().enumerate() {
//!         PyTuple_SET_ITEM(
//!             pyall,
//!             i as isize,
//!             PyUnicode_InternFromString(obj.as_ptr() as *const c_char),
//!         )
//!     }
//!
//!     PyModule_AddObject(mptr, "__all__\0".as_ptr() as *const c_char, pyall);
//!
//!     mptr
//! }
//!
//! pub unsafe extern "C" fn sum_as_string(
//!     _self: *mut PyObject,
//!     args: *mut *mut PyObject,
//!     nargs: Py_ssize_t,
//! ) -> *mut PyObject {
//!     if nargs != 2 {
//!         return raise_type_error("sum_as_string() expected 2 positional arguments");
//!     }
//!
//!     let arg1 = *args;
//!     if PyLong_Check(arg1) == 0 {
//!         return raise_type_error("sum_as_string() expected an int for positional argument 1");
//!     }
//!
//!     let arg1 = PyLong_AsLong(arg1);
//!     if !PyErr_Occurred().is_null() {
//!         return ptr::null_mut()
//!     }
//!
//!     let arg2 = *args.add(1);
//!     if PyLong_Check(arg2) == 0 {
//!         return raise_type_error("sum_as_string() expected an int for positional argument 2");
//!     }
//!
//!     let arg2 = PyLong_AsLong(arg2);
//!     if !PyErr_Occurred().is_null() {
//!         return ptr::null_mut()
//!     }
//!     let res = (arg1 + arg2).to_string();
//!     PyUnicode_FromStringAndSize(res.as_ptr() as *const c_char, res.len() as isize)
//! }
//!
//! #[cold]
//! #[inline(never)]
//! fn raise_type_error(msg: &str) -> *mut PyObject {
//!     unsafe {
//!         let err_msg =
//!             PyUnicode_FromStringAndSize(msg.as_ptr() as *const c_char, msg.len() as isize);
//!         PyErr_SetObject(PyExc_TypeError, err_msg);
//!         Py_DECREF(err_msg);
//!     };
//!     std::ptr::null_mut()
//! }
//! ```
//!
//! With those two files in place, now `maturin` needs to be installed. This can be done using
//! Python's package manager `pip`. First, load up a new Python `virtualenv`, and install `maturin`
//! into it:
//! ```bash
//! $ cd string_sum
//! $ python -m venv .env
//! $ source .env/bin/activate
//! $ pip install maturin
//! ```
//!
//! Now build and execute the module:
//! ```bash
//! $ maturin develop
//! # lots of progress output as maturin runs the compilation...
//! $ python
//! >>> import string_sum
//! >>> string_sum.sum_as_string(5, 20)
//! '25'
//! ```
//!
//! As well as with `maturin`, it is possible to build using [setuptools-rust] or
//! [manually][manual_builds]. Both offer more flexibility than `maturin` but require further
//! configuration.
//!
//!
//! # Using Python from Rust
//!
//! To embed Python into a Rust binary, you need to ensure that your Python installation contains a
//! shared library. The following steps demonstrate how to ensure this (for Ubuntu).
//!
//! To install the Python shared library on Ubuntu:
//! ```bash
//! sudo apt install python3-dev
//! ```
//!
//! While most projects use the safe wrapper provided by pyo3,
//! you can take a look at the [`orjson`] library as an example on how to use `pyo3-ffi` directly.
//! For those well versed in C and Rust the [tutorials] from the CPython documentation
//! can be easily converted to rust as well.
//!
//! [tutorials]: https://docs.python.org/3/extending/
//! [`orjson`]: https://github.com/ijl/orjson
//! [capi]: https://docs.python.org/3/c-api/index.html
//! [`maturin`]: https://github.com/PyO3/maturin "Build and publish crates with pyo3, rust-cpython and cffi bindings as well as rust binaries as python packages"
//! [`pyo3-build-config`]: https://docs.rs/pyo3-build-config
//! [feature flags]: https://doc.rust-lang.org/cargo/reference/features.html "Features - The Cargo Book"
//! [manual_builds]: https://pyo3.rs/latest/building_and_distribution.html#manual-builds "Manual builds - Building and Distribution - PyO3 user guide"
//! [setuptools-rust]: https://github.com/PyO3/setuptools-rust "Setuptools plugin for Rust extensions"
//! [PEP 384]: https://www.python.org/dev/peps/pep-0384 "PEP 384 -- Defining a Stable ABI"
//! [Features chapter of the guide]: https://pyo3.rs/latest/features.html#features-reference "Features Reference - PyO3 user guide"

#![allow(
    missing_docs,
    non_camel_case_types,
    non_snake_case,
    non_upper_case_globals,
    clippy::upper_case_acronyms,
    clippy::missing_safety_doc
)]

// Until `extern type` is stabilized, use the recommended approach to
// model opaque types:
// https://doc.rust-lang.org/nomicon/ffi.html#representing-opaque-structs
macro_rules! opaque_struct {
    ($name:ident) => {
        #[repr(C)]
        pub struct $name([u8; 0]);
    };
}

pub use self::abstract_::*;
pub use self::bltinmodule::*;
pub use self::boolobject::*;
#[cfg(Py_3_11)]
pub use self::buffer::*;
pub use self::bytearrayobject::*;
pub use self::bytesobject::*;
pub use self::ceval::*;
pub use self::code::*;
pub use self::codecs::*;
pub use self::compile::*;
pub use self::complexobject::*;
#[cfg(all(Py_3_8, not(Py_LIMITED_API)))]
pub use self::context::*;
#[cfg(not(Py_LIMITED_API))]
pub use self::datetime::*;
pub use self::descrobject::*;
pub use self::dictobject::*;
pub use self::enumobject::*;
pub use self::eval::*;
pub use self::fileobject::*;
pub use self::fileutils::*;
pub use self::floatobject::*;
#[cfg(not(Py_LIMITED_API))]
pub use self::funcobject::*;
#[cfg(not(Py_LIMITED_API))]
pub use self::genobject::*;
pub use self::import::*;
pub use self::intrcheck::*;
pub use self::iterobject::*;
pub use self::listobject::*;
pub use self::longobject::*;
pub use self::marshal::*;
pub use self::memoryobject::*;
pub use self::methodobject::*;
pub use self::modsupport::*;
pub use self::moduleobject::*;
pub use self::object::*;
pub use self::objimpl::*;
pub use self::osmodule::*;
#[cfg(not(Py_LIMITED_API))]
pub use self::pyarena::*;
pub use self::pycapsule::*;
pub use self::pyerrors::*;
pub use self::pyframe::*;
pub use self::pyhash::*;
pub use self::pylifecycle::*;
pub use self::pymem::*;
pub use self::pyport::*;
pub use self::pystate::*;
pub use self::pystrtod::*;
pub use self::pythonrun::*;
pub use self::rangeobject::*;
pub use self::setobject::*;
pub use self::sliceobject::*;
pub use self::structseq::*;
pub use self::sysmodule::*;
pub use self::traceback::*;
pub use self::tupleobject::*;
pub use self::typeslots::*;
pub use self::unicodeobject::*;
pub use self::warnings::*;
pub use self::weakrefobject::*;

mod abstract_;
// skipped asdl.h
// skipped ast.h
mod bltinmodule;
mod boolobject;
#[cfg(Py_3_11)]
mod buffer;
mod bytearrayobject;
mod bytesobject;
// skipped cellobject.h
mod ceval;
// skipped classobject.h
mod code;
mod codecs;
mod compile;
mod complexobject;
#[cfg(all(Py_3_8, not(Py_LIMITED_API)))]
mod context; // It's actually 3.7.1, but no cfg for patches.
#[cfg(not(Py_LIMITED_API))]
pub(crate) mod datetime;
mod descrobject;
mod dictobject;
// skipped dynamic_annotations.h
mod enumobject;
// skipped errcode.h
mod eval;
// skipped exports.h
mod fileobject;
mod fileutils;
mod floatobject;
// skipped empty frameobject.h
#[cfg(not(Py_LIMITED_API))]
pub(crate) mod funcobject;
// skipped genericaliasobject.h
#[cfg(not(Py_LIMITED_API))]
mod genobject;
mod import;
// skipped interpreteridobject.h
mod intrcheck;
mod iterobject;
mod listobject;
// skipped longintrepr.h
mod longobject;
pub(crate) mod marshal;
mod memoryobject;
mod methodobject;
mod modsupport;
mod moduleobject;
// skipped namespaceobject.h
mod object;
mod objimpl;
// skipped odictobject.h
// skipped opcode.h
// skipped osdefs.h
mod osmodule;
// skipped parser_interface.h
// skipped patchlevel.h
// skipped picklebufobject.h
// skipped pyctype.h
// skipped py_curses.h
#[cfg(not(Py_LIMITED_API))]
mod pyarena;
mod pycapsule;
// skipped pydecimal.h
// skipped pydtrace.h
mod pyerrors;
// skipped pyexpat.h
// skipped pyfpe.h
mod pyframe;
mod pyhash;
mod pylifecycle;
// skipped pymacconfig.h
// skipped pymacro.h
// skipped pymath.h
mod pymem;
mod pyport;
mod pystate;
mod pythonrun;
// skipped pystrhex.h
// skipped pystrcmp.h
mod pystrtod;
// skipped pythread.h
// skipped pytime.h
mod rangeobject;
mod setobject;
mod sliceobject;
mod structseq;
mod sysmodule;
mod traceback;
// skipped tracemalloc.h
mod tupleobject;
mod typeslots;
mod unicodeobject;
mod warnings;
mod weakrefobject;

// Additional headers that are not exported by Python.h
pub mod structmember;

// "Limited API" definitions matching Python's `include/cpython` directory.
#[cfg(not(Py_LIMITED_API))]
mod cpython;

#[cfg(not(Py_LIMITED_API))]
pub use self::cpython::*;
