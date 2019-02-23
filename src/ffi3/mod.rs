//! Rust FFI declarations for Python 3
#![allow(non_camel_case_types, non_snake_case, non_upper_case_globals)]
#![cfg_attr(Py_LIMITED_API, allow(unused_imports))]
#![cfg_attr(feature="cargo-clippy", allow(clippy::inline_always))]

pub use self::bltinmodule::*;
pub use self::boolobject::*;
pub use self::bytearrayobject::*;
pub use self::bytesobject::*;
pub use self::ceval::*;
pub use self::code::*;
pub use self::codecs::*;
pub use self::compile::*;
pub use self::complexobject::*;
pub use self::descrobject::*;
pub use self::dictobject::*;
pub use self::enumobject::*;
pub use self::eval::*;
pub use self::fileobject::*;
pub use self::floatobject::*;
pub use self::frameobject::PyFrameObject;
pub use self::genobject::*;
pub use self::import::*;
pub use self::intrcheck::*;
pub use self::iterobject::*;
pub use self::listobject::*;
pub use self::longobject::*;
pub use self::memoryobject::*;
pub use self::methodobject::*;
pub use self::modsupport::*;
pub use self::moduleobject::*;
pub use self::object::*;
pub use self::objectabstract::*;
pub use self::objimpl::*;
#[cfg(Py_3_6)]
pub use self::osmodule::*;
pub use self::pyarena::*;
pub use self::pycapsule::*;
pub use self::pydebug::*;
pub use self::pyerrors::*;
pub use self::pyhash::*;
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

mod pyport;
// mod pymacro; contains nothing of interest for Rust
// mod pyatomic; contains nothing of interest for Rust
// mod pymath; contains nothing of interest for Rust

// [cfg(not(Py_LIMITED_API))]
// mod pytime; contains nothing of interest

mod pymem;

mod object;
mod objimpl;
mod pydebug;
mod pyhash;
mod typeslots;

mod bytearrayobject;
mod bytesobject;
mod longobject;
mod unicodeobject; // TODO supports PEP-384 only; needs adjustment for Python 3.3 and 3.5
                   // mod longintrepr; TODO excluded by PEP-384
mod boolobject; // TODO supports PEP-384 only; needs adjustment for Python 3.3 and 3.5
mod complexobject; // TODO supports PEP-384 only; needs adjustment for Python 3.3 and 3.5
mod dictobject;
mod floatobject; // TODO supports PEP-384 only; needs adjustment for Python 3.3 and 3.5
mod listobject; // TODO supports PEP-384 only; needs adjustment for Python 3.3 and 3.5
mod memoryobject; // TODO supports PEP-384 only; needs adjustment for Python 3.3 and 3.5
mod rangeobject; // TODO supports PEP-384 only; needs adjustment for Python 3.3 and 3.5
mod tupleobject; // TODO supports PEP-384 only; needs adjustment for Python 3.3 and 3.5
                 // mod odictobject; TODO new in 3.5
mod enumobject; // TODO supports PEP-384 only; needs adjustment for Python 3.3 and 3.5
mod methodobject; // TODO supports PEP-384 only; needs adjustment for Python 3.3 and 3.5
mod moduleobject;
mod setobject; // TODO supports PEP-384 only; needs adjustment for Python 3.3 and 3.5
               // mod funcobject; TODO excluded by PEP-384
               // mod classobject; TODO excluded by PEP-384
mod fileobject; // TODO supports PEP-384 only; needs adjustment for Python 3.3 and 3.5
mod pycapsule; // TODO supports PEP-384 only; needs adjustment for Python 3.3 and 3.5
mod sliceobject;
mod traceback; // TODO supports PEP-384 only; needs adjustment for Python 3.3 and 3.5
               // mod cellobject; TODO excluded by PEP-384
mod descrobject; // TODO supports PEP-384 only; needs adjustment for Python 3.3 and 3.5
mod genobject; // TODO excluded by PEP-384
mod iterobject; // TODO supports PEP-384 only; needs adjustment for Python 3.3 and 3.5
mod structseq;
mod warnings; // TODO supports PEP-384 only; needs adjustment for Python 3.3 and 3.5
mod weakrefobject; // TODO supports PEP-384 only; needs adjustment for Python 3.3 and 3.5
                   // mod namespaceobject; TODO

mod codecs; // TODO supports PEP-384 only; needs adjustment for Python 3.3 and 3.5
mod pyerrors; // TODO supports PEP-384 only; needs adjustment for Python 3.3 and 3.5

mod pystate; // TODO supports PEP-384 only; needs adjustment for Python 3.3 and 3.5

#[cfg(Py_LIMITED_API)]
mod pyarena {}
mod modsupport; // TODO supports PEP-384 only; needs adjustment for Python 3.3 and 3.5
#[cfg(not(Py_LIMITED_API))]
mod pyarena; // TODO: incomplete
mod pythonrun; // TODO some functions need to be moved to pylifecycle
               //mod pylifecycle; // TODO new in 3.5
mod ceval; // TODO supports PEP-384 only; needs adjustment for Python 3.3 and 3.5
mod import;
mod intrcheck; // TODO supports PEP-384 only; needs adjustment for Python 3.3 and 3.5
#[cfg(Py_3_6)]
mod osmodule;
mod sysmodule; // TODO supports PEP-384 only; needs adjustment for Python 3.3 and 3.5

mod bltinmodule;
mod objectabstract; // TODO supports PEP-384 only; needs adjustment for Python 3.3 and 3.5

#[cfg(Py_LIMITED_API)]
mod code {}
#[cfg(not(Py_LIMITED_API))]
mod code;

mod compile; // TODO: incomplete
mod eval; // TODO supports PEP-384 only; needs adjustment for Python 3.3 and 3.5

// mod pyctype; TODO excluded by PEP-384
mod pystrtod; // TODO supports PEP-384 only; needs adjustment for Python 3.3 and 3.5
              // mod pystrcmp; TODO nothing interesting for Rust?
              // mod dtoa; TODO excluded by PEP-384
              // mod fileutils; TODO no public functions?
              // mod pyfpe; TODO probably not interesting for rust

// Additional headers that are not exported by Python.h
pub mod structmember; // TODO supports PEP-384 only; needs adjustment for Python 3.3 and 3.5

#[cfg(not(Py_LIMITED_API))]
pub mod frameobject;
#[cfg(Py_LIMITED_API)]
pub mod frameobject {
    pub enum PyFrameObject {}
}
