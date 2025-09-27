#![cfg_attr(docsrs, feature(doc_cfg, doc_auto_cfg))]
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
//! - `Py_3_7`, `Py_3_8`, `Py_3_9`, `Py_3_10`, `Py_3_11`, `Py_3_12`, `Py_3_13`: Marks code that is
//!    only enabled when compiling for a given minimum Python version.
//! - `Py_LIMITED_API`: Marks code enabled when the `abi3` feature flag is enabled.
//! - `Py_GIL_DISABLED`: Marks code that runs only in the free-threaded build of CPython.
//! - `PyPy` - Marks code enabled when compiling for PyPy.
//! - `GraalPy` - Marks code enabled when compiling for GraalPy.
//!
//! Additionally, you can query for the values `Py_DEBUG`, `Py_REF_DEBUG`,
//! `Py_TRACE_REFS`, and `COUNT_ALLOCS` from `py_sys_config` to query for the
//! corresponding C build-time defines. For example, to conditionally define
//! debug code using `Py_DEBUG`, you could do:
//!
//! ```rust,ignore
//! #[cfg(py_sys_config = "Py_DEBUG")]
//! println!("only runs if python was compiled with Py_DEBUG")
//! ```
//!
//! To use these attributes, add [`pyo3-build-config`] as a build dependency in
//! your `Cargo.toml`:
//!
//! ```toml
//! [build-dependencies]
#![doc = concat!("pyo3-build-config =\"", env!("CARGO_PKG_VERSION"),  "\"")]
//! ```
//!
//! And then either create a new `build.rs` file in the project root or modify
//! the existing `build.rs` file to call `use_pyo3_cfgs()`:
//!
//! ```rust,ignore
//! fn main() {
//!     pyo3_build_config::use_pyo3_cfgs();
//! }
//! ```
//!
//! # Minimum supported Rust and Python versions
//!
//! `pyo3-ffi` supports the following Python distributions:
//!   - CPython 3.7 or greater
//!   - PyPy 7.3 (Python 3.9+)
//!   - GraalPy 24.0 or greater (Python 3.10+)
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
#![doc = concat!("version = \"", env!("CARGO_PKG_VERSION"),  "\"")]
//! features = ["extension-module"]
//!
//! [build-dependencies]
//! # This is only necessary if you need to configure your build based on
//! # the Python version or the compile-time configuration for the interpreter.
#![doc = concat!("pyo3_build_config = \"", env!("CARGO_PKG_VERSION"),  "\"")]
//! ```
//!
//! If you need to use conditional compilation based on Python version or how
//! Python was compiled, you need to add `pyo3-build-config` as a
//! `build-dependency` in your `Cargo.toml` as in the example above and either
//! create a new `build.rs` file or modify an existing one so that
//! `pyo3_build_config::use_pyo3_cfgs()` gets called at build time:
//!
//! **`build.rs`**
//! ```rust,ignore
//! fn main() {
//!     pyo3_build_config::use_pyo3_cfgs()
//! }
//! ```
//!
//! **`src/lib.rs`**
//! ```rust,no_run
//! use std::ffi::{c_char, c_long};
//! use std::ptr;
//!
//! use pyo3_ffi::*;
//!
//! static mut MODULE_DEF: PyModuleDef = PyModuleDef {
//!     m_base: PyModuleDef_HEAD_INIT,
//!     m_name: c_str!("string_sum").as_ptr(),
//!     m_doc: c_str!("A Python module written in Rust.").as_ptr(),
//!     m_size: 0,
//!     m_methods: unsafe { METHODS as *const [PyMethodDef] as *mut PyMethodDef },
//!     m_slots: std::ptr::null_mut(),
//!     m_traverse: None,
//!     m_clear: None,
//!     m_free: None,
//! };
//!
//! static mut METHODS: &[PyMethodDef] = &[
//!     PyMethodDef {
//!         ml_name: c_str!("sum_as_string").as_ptr(),
//!         ml_meth: PyMethodDefPointer {
//!             PyCFunctionFast: sum_as_string,
//!         },
//!         ml_flags: METH_FASTCALL,
//!         ml_doc: c_str!("returns the sum of two integers as a string").as_ptr(),
//!     },
//!     // A zeroed PyMethodDef to mark the end of the array.
//!     PyMethodDef::zeroed(),
//! ];
//!
//! // The module initialization function, which must be named `PyInit_<your_module>`.
//! #[allow(non_snake_case)]
//! #[no_mangle]
//! pub unsafe extern "C" fn PyInit_string_sum() -> *mut PyObject {
//!     let module = PyModule_Create(ptr::addr_of_mut!(MODULE_DEF));
//!     if module.is_null() {
//!         return module;
//!     }
//!     #[cfg(Py_GIL_DISABLED)]
//!     {
//!         if PyUnstable_Module_SetGIL(module, Py_MOD_GIL_NOT_USED) < 0 {
//!             Py_DECREF(module);
//!             return std::ptr::null_mut();
//!         }
//!     }
//!     module
//! }
//!
//! /// A helper to parse function arguments
//! /// If we used PyO3's proc macros they'd handle all of this boilerplate for us :)
//! unsafe fn parse_arg_as_i32(obj: *mut PyObject, n_arg: usize) -> Option<i32> {
//!     if PyLong_Check(obj) == 0 {
//!         let msg = format!(
//!             "sum_as_string expected an int for positional argument {}\0",
//!             n_arg
//!         );
//!         PyErr_SetString(PyExc_TypeError, msg.as_ptr().cast::<c_char>());
//!         return None;
//!     }
//!
//!     // Let's keep the behaviour consistent on platforms where `c_long` is bigger than 32 bits.
//!     // In particular, it is an i32 on Windows but i64 on most Linux systems
//!     let mut overflow = 0;
//!     let i_long: c_long = PyLong_AsLongAndOverflow(obj, &mut overflow);
//!
//!     #[allow(irrefutable_let_patterns)] // some platforms have c_long equal to i32
//!     if overflow != 0 {
//!         raise_overflowerror(obj);
//!         None
//!     } else if let Ok(i) = i_long.try_into() {
//!         Some(i)
//!     } else {
//!         raise_overflowerror(obj);
//!         None
//!     }
//! }
//!
//! unsafe fn raise_overflowerror(obj: *mut PyObject) {
//!     let obj_repr = PyObject_Str(obj);
//!     if !obj_repr.is_null() {
//!         let mut size = 0;
//!         let p = PyUnicode_AsUTF8AndSize(obj_repr, &mut size);
//!         if !p.is_null() {
//!             let s = std::str::from_utf8_unchecked(std::slice::from_raw_parts(
//!                 p.cast::<u8>(),
//!                 size as usize,
//!             ));
//!             let msg = format!("cannot fit {} in 32 bits\0", s);
//!
//!             PyErr_SetString(PyExc_OverflowError, msg.as_ptr().cast::<c_char>());
//!         }
//!         Py_DECREF(obj_repr);
//!     }
//! }
//!
//! pub unsafe extern "C" fn sum_as_string(
//!     _self: *mut PyObject,
//!     args: *mut *mut PyObject,
//!     nargs: Py_ssize_t,
//! ) -> *mut PyObject {
//!     if nargs != 2 {
//!         PyErr_SetString(
//!             PyExc_TypeError,
//!             c_str!("sum_as_string expected 2 positional arguments").as_ptr(),
//!         );
//!         return std::ptr::null_mut();
//!     }
//!
//!     let (first, second) = (*args, *args.add(1));
//!
//!     let first = match parse_arg_as_i32(first, 1) {
//!         Some(x) => x,
//!         None => return std::ptr::null_mut(),
//!     };
//!     let second = match parse_arg_as_i32(second, 2) {
//!         Some(x) => x,
//!         None => return std::ptr::null_mut(),
//!     };
//!
//!     match first.checked_add(second) {
//!         Some(sum) => {
//!             let string = sum.to_string();
//!             PyUnicode_FromStringAndSize(string.as_ptr().cast::<c_char>(), string.len() as isize)
//!         }
//!         None => {
//!             PyErr_SetString(
//!                 PyExc_OverflowError,
//!                 c_str!("arguments too large to add").as_ptr(),
//!             );
//!             std::ptr::null_mut()
//!         }
//!     }
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
//! This example stores the module definition statically and uses the `PyModule_Create` function
//! in the CPython C API to register the module. This is the "old" style for registering modules
//! and has the limitation that it cannot support subinterpreters. You can also create a module
//! using the new multi-phase initialization API that does support subinterpreters. See the
//! `sequential` project located in the `examples` directory at the root of the `pyo3-ffi` crate
//! for a worked example of how to this using `pyo3-ffi`.
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
#![doc = concat!("[manual_builds]: https://pyo3.rs/v", env!("CARGO_PKG_VERSION"), "/building-and-distribution.html#manual-builds \"Manual builds - Building and Distribution - PyO3 user guide\"")]
//! [setuptools-rust]: https://github.com/PyO3/setuptools-rust "Setuptools plugin for Rust extensions"
//! [PEP 384]: https://www.python.org/dev/peps/pep-0384 "PEP 384 -- Defining a Stable ABI"
#![doc = concat!("[Features chapter of the guide]: https://pyo3.rs/v", env!("CARGO_PKG_VERSION"), "/features.html#features-reference \"Features eference - PyO3 user guide\"")]
#![allow(
    missing_docs,
    non_camel_case_types,
    non_snake_case,
    non_upper_case_globals,
    clippy::upper_case_acronyms,
    clippy::missing_safety_doc,
    clippy::ptr_eq
)]
#![warn(elided_lifetimes_in_paths, unused_lifetimes)]
// This crate is a hand-maintained translation of CPython's headers, so requiring "unsafe"
// blocks within those translations increases maintenance burden without providing any
// additional safety. The safety of the functions in this crate is determined by the
// original CPython headers
#![allow(unsafe_op_in_unsafe_fn)]

// Until `extern type` is stabilized, use the recommended approach to
// model opaque types:
// https://doc.rust-lang.org/nomicon/ffi.html#representing-opaque-structs
macro_rules! opaque_struct {
    ($(#[$attrs:meta])* $pub:vis $name:ident) => {
        $(#[$attrs])*
        #[repr(C)]
        $pub struct $name([u8; 0]);
    };
}

/// This is a helper macro to create a `&'static CStr`.
///
/// It can be used on all Rust versions supported by PyO3, unlike c"" literals which
/// were stabilised in Rust 1.77.
///
/// Due to the nature of PyO3 making heavy use of C FFI interop with Python, it is
/// common for PyO3 to use CStr.
///
/// Examples:
///
/// ```rust,no_run
/// use std::ffi::CStr;
///
/// const HELLO: &CStr = pyo3_ffi::c_str!("hello");
/// static WORLD: &CStr = pyo3_ffi::c_str!("world");
/// ```
#[macro_export]
macro_rules! c_str {
    ($s:expr) => {
        $crate::_cstr_from_utf8_with_nul_checked(concat!($s, "\0"))
    };
}

/// Private helper for `c_str!` macro.
#[doc(hidden)]
pub const fn _cstr_from_utf8_with_nul_checked(s: &str) -> &std::ffi::CStr {
    match std::ffi::CStr::from_bytes_with_nul(s.as_bytes()) {
        Ok(cstr) => cstr,
        Err(_) => panic!("string contains nul bytes"),
    }
}

pub mod compat;
mod impl_;

pub use self::abstract_::*;
pub use self::bltinmodule::*;
pub use self::boolobject::*;
pub use self::bytearrayobject::*;
pub use self::bytesobject::*;
pub use self::ceval::*;
#[cfg(Py_LIMITED_API)]
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
pub use self::fileobject::*;
pub use self::fileutils::*;
pub use self::floatobject::*;
#[cfg(Py_3_9)]
pub use self::genericaliasobject::*;
pub use self::import::*;
pub use self::intrcheck::*;
pub use self::iterobject::*;
pub use self::listobject::*;
pub use self::longobject::*;
#[cfg(not(Py_LIMITED_API))]
pub use self::marshal::*;
pub use self::memoryobject::*;
pub use self::methodobject::*;
pub use self::modsupport::*;
pub use self::moduleobject::*;
pub use self::object::*;
pub use self::objimpl::*;
pub use self::osmodule::*;
#[cfg(not(any(PyPy, Py_LIMITED_API, Py_3_10)))]
pub use self::pyarena::*;
#[cfg(Py_3_11)]
pub use self::pybuffer::*;
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
pub use self::pytypedefs::*;
pub use self::rangeobject::*;
pub use self::refcount::*;
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
mod bytearrayobject;
mod bytesobject;
// skipped cellobject.h
mod ceval;
// skipped classobject.h
#[cfg(Py_LIMITED_API)]
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
// skipped exports.h
mod fileobject;
mod fileutils;
mod floatobject;
// skipped empty frameobject.h
mod genericaliasobject;
mod import;
// skipped interpreteridobject.h
mod intrcheck;
mod iterobject;
mod listobject;
// skipped longintrepr.h
mod longobject;
#[cfg(not(Py_LIMITED_API))]
pub mod marshal;
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
#[cfg(not(any(PyPy, Py_LIMITED_API, Py_3_10)))]
mod pyarena;
#[cfg(Py_3_11)]
mod pybuffer;
mod pycapsule;
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
// skipped pystats.h
mod pythonrun;
// skipped pystrhex.h
// skipped pystrcmp.h
mod pystrtod;
// skipped pythread.h
// skipped pytime.h
mod pytypedefs;
mod rangeobject;
mod refcount;
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
#[deprecated(note = "Python 3.12")]
pub mod structmember;

// "Limited API" definitions matching Python's `include/cpython` directory.
#[cfg(not(Py_LIMITED_API))]
mod cpython;

#[cfg(not(Py_LIMITED_API))]
pub use self::cpython::*;
