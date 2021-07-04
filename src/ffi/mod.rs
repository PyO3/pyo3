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

#[cfg(not(Py_LIMITED_API))]
pub use self::cpython::*;

mod abstract_;
// skipped asdl.h
// skipped ast.h
mod bltinmodule;
mod boolobject; // TODO supports PEP-384 only
mod bytearrayobject;
mod bytesobject;
// skipped cellobject.h
mod ceval; // TODO supports PEP-384 only

// skipped classobject.h
mod code;
mod codecs; // TODO supports PEP-384 only
mod compile; // TODO: incomplete
mod complexobject; // TODO supports PEP-384 only
#[cfg(all(Py_3_8, not(Py_LIMITED_API)))]
mod context; // It's actually 3.7.1, but no cfg for patches.
#[cfg(not(Py_LIMITED_API))]
pub(crate) mod datetime;
mod descrobject; // TODO supports PEP-384 only
mod dictobject;
// skipped dynamic_annotations.h
mod enumobject;
// skipped errcode.h
mod eval; // TODO supports PEP-384 only

// skipped exports.h
mod fileobject; // TODO: incomplete

// skipped fileutils.h
mod floatobject; // TODO supports PEP-384 only

// skipped empty frameobject.h
#[cfg(not(Py_LIMITED_API))]
pub(crate) mod funcobject;
// skipped genericaliasobject.h
#[cfg(not(Py_LIMITED_API))]
mod genobject; // TODO: incomplete
mod import; // TODO: incomplete

// skipped interpreteridobject.h
mod intrcheck; // TODO supports PEP-384 only
mod iterobject;
mod listobject;
// skipped longintrepr.h
mod longobject;
pub(crate) mod marshal;
mod memoryobject;
mod methodobject; // TODO: incomplete
mod modsupport; // TODO: incomplete
mod moduleobject; // TODO: incomplete

// skipped namespaceobject.h
mod object;
// skipped odictobject.h
// skipped opcode.h
// skipped osdefs.h
// skipped parser_interface.h
// skipped patchlevel.h
// skipped picklebufobject.h
// skipped pyctype.h
// skipped py_curses.h
// skipped pydecimal.h
// skipped pydtrace.h
// skipped pyexpat.h
// skipped pyfpe.h
mod pyframe; // TODO: incomplete

// skipped pymacconfig.h
// skipped pymacro.h
// skipped pymath.h
// skipped pystrcmp.h
// skipped pystrhex.h
// skipped Python-ast.h
// this file is Python.h
// skipped pythread.h
// skipped pytime.h

mod pyport;
// mod pymacro; contains nothing of interest for Rust
// mod pyatomic; contains nothing of interest for Rust
// mod pymath; contains nothing of interest for Rust

// [cfg(not(Py_LIMITED_API))]
// mod pytime; contains nothing of interest

mod objimpl;
mod pyhash;
mod pymem;
mod typeslots;

mod unicodeobject; // TODO supports PEP-384 only; needs adjustment for Python 3.3 and 3.5
                   // mod longintrepr; TODO excluded by PEP-384
mod rangeobject; // TODO supports PEP-384 only; needs adjustment for Python 3.3 and 3.5
mod tupleobject; // TODO supports PEP-384 only; needs adjustment for Python 3.3 and 3.5
                 // mod odictobject; TODO new in 3.5
mod setobject; // TODO supports PEP-384 only; needs adjustment for Python 3.3 and 3.5
               // mod classobject; TODO excluded by PEP-384
mod pycapsule; // TODO supports PEP-384 only; needs adjustment for Python 3.3 and 3.5
mod sliceobject;
mod structseq;
mod traceback; // TODO supports PEP-384 only; needs adjustment for Python 3.3 and 3.5
mod warnings; // TODO supports PEP-384 only; needs adjustment for Python 3.3 and 3.5
mod weakrefobject; // TODO supports PEP-384 only; needs adjustment for Python 3.3 and 3.5

mod pyerrors; // TODO supports PEP-384 only; needs adjustment for Python 3.3 and 3.5
mod pylifecycle;
mod pystate; // TODO supports PEP-384 only; needs adjustment for Python 3.3 and 3.5

#[cfg(Py_LIMITED_API)]
mod pyarena {}
#[cfg(not(Py_LIMITED_API))]
mod pyarena; // TODO: incomplete
mod pythonrun; // TODO some functions need to be moved to pylifecycle
               //mod pylifecycle; // TODO new in 3.5
mod osmodule;
mod sysmodule; // TODO supports PEP-384 only; needs adjustment for Python 3.3 and 3.5

// mod pyctype; TODO excluded by PEP-384
mod pystrtod; // TODO supports PEP-384 only; needs adjustment for Python 3.3 and 3.5
              // mod pystrcmp; TODO nothing interesting for Rust?
              // mod dtoa; TODO excluded by PEP-384
              // mod fileutils; TODO no public functions?
              // mod pyfpe; TODO probably not interesting for rust

// Additional headers that are not exported by Python.h
pub mod structmember; // TODO supports PEP-384 only; needs adjustment for Python 3.3 and 3.5

#[cfg(not(Py_LIMITED_API))]
mod cpython;
