//! Raw FFI declarations for Python's C API.
//!
//! This module provides low level bindings to the Python interpreter.
//! It is meant for advanced users only - regular PyO3 users shouldn't
//! need to interact with this module at all.
//!
//! The contents of this module are not documented here, as it would entail
//! basically copying the documentation from CPython. Consult the [Python/C API Reference
//! Manual][capi] for up-to-date documentation.
//!
//! # Safety
//!
//! The functions in this module lack individual safety documentation, but
//! generally the following apply:
//! - Pointer arguments have to point to a valid Python object of the correct type,
//! although null pointers are sometimes valid input.
//! - The vast majority can only be used safely while the GIL is held.
//! - Some functions have additional safety requirements, consult the
//! [Python/C API Reference Manual][capi]
//! for more information.
//!
//! [capi]: https://docs.python.org/3/c-api/index.html
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

/// Helper to enable #\[pymethods\] to see the workaround for __ipow__ on Python 3.7
#[doc(hidden)]
pub use crate::impl_::pymethods::ipowfunc;
