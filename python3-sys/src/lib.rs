#![allow(non_camel_case_types, non_snake_case, non_upper_case_globals, raw_pointer_derive)]
#![cfg_attr(Py_LIMITED_API, allow(unused_imports))]

// old: marked with TODO
// Based on the headers of Python 3.4.3
// Supports the stable ABI (PEP 384) only.

// new:
// Based on the headers of Python 3.3.0, 3.4.0 and 3.5.0.

extern crate libc;

pub use pyport::*;
pub use pymem::*;

pub use object::*;
pub use objimpl::*;
pub use typeslots::*;

pub use bytearrayobject::*;
pub use bytesobject::*;
pub use unicodeobject::*;
pub use longobject::*;
pub use boolobject::*;
pub use floatobject::*;
pub use complexobject::*;
pub use rangeobject::*;
pub use memoryobject::*;
pub use tupleobject::*;
pub use listobject::*;
pub use dictobject::*;
pub use enumobject::*;
pub use setobject::*;
pub use methodobject::*;
pub use moduleobject::*;
pub use fileobject::*;
pub use pycapsule::*;
pub use traceback::*;
pub use sliceobject::*;
pub use iterobject::*;
pub use descrobject::*;
pub use warnings::*;
pub use weakrefobject::*;
pub use structseq::*;

pub use codecs::*;
pub use pyerrors::*;

pub use pystate::*;

pub use pyarena::*;
pub use modsupport::*;
pub use pythonrun::*;
pub use ceval::*;
pub use sysmodule::*;
pub use intrcheck::*;
pub use import::*;

pub use objectabstract::*;
pub use bltinmodule::*;

pub use code::*;
pub use compile::*;
pub use eval::*;

pub use pystrtod::*;

mod pyport;
// mod pymacro; contains nothing of interest for Rust

// mod pyatomic; contains nothing of interest for Rust

// mod pymath; contains nothing of interest for Rust

// [cfg(not(Py_LIMITED_API))]
// mod pytime; contains nothing of interest

mod pymem;

mod object;
mod objimpl; // TODO supports PEP-384 only; needs adjustment for Python 3.3 and 3.5
mod typeslots; // TODO supports PEP-384 only; needs adjustment for Python 3.3 and 3.5
// mod pyhash; new in 3.4; contains nothing of interest

// mod pydebug; TODO excluded by PEP-384

mod bytearrayobject; // TODO supports PEP-384 only; needs adjustment for Python 3.3 and 3.5
mod bytesobject; // TODO supports PEP-384 only; needs adjustment for Python 3.3 and 3.5
mod unicodeobject; // TODO supports PEP-384 only; needs adjustment for Python 3.3 and 3.5
mod longobject; // TODO supports PEP-384 only; needs adjustment for Python 3.3 and 3.5
// mod longintrepr; TODO excluded by PEP-384
mod boolobject; // TODO supports PEP-384 only; needs adjustment for Python 3.3 and 3.5
mod floatobject; // TODO supports PEP-384 only; needs adjustment for Python 3.3 and 3.5
mod complexobject; // TODO supports PEP-384 only; needs adjustment for Python 3.3 and 3.5
mod rangeobject; // TODO supports PEP-384 only; needs adjustment for Python 3.3 and 3.5
mod memoryobject; // TODO supports PEP-384 only; needs adjustment for Python 3.3 and 3.5
mod tupleobject;
mod listobject; // TODO supports PEP-384 only; needs adjustment for Python 3.3 and 3.5
mod dictobject; // TODO supports PEP-384 only; needs adjustment for Python 3.3 and 3.5
// mod odictobject; TODO new in 3.5
mod enumobject; // TODO supports PEP-384 only; needs adjustment for Python 3.3 and 3.5
mod setobject; // TODO supports PEP-384 only; needs adjustment for Python 3.3 and 3.5
mod methodobject; // TODO supports PEP-384 only; needs adjustment for Python 3.3 and 3.5
mod moduleobject; // TODO supports PEP-384 only; needs adjustment for Python 3.3 and 3.5
// mod funcobject; TODO excluded by PEP-384
// mod classobject; TODO excluded by PEP-384
mod fileobject; // TODO supports PEP-384 only; needs adjustment for Python 3.3 and 3.5
mod pycapsule; // TODO supports PEP-384 only; needs adjustment for Python 3.3 and 3.5
mod traceback; // TODO supports PEP-384 only; needs adjustment for Python 3.3 and 3.5
mod sliceobject; // TODO supports PEP-384 only; needs adjustment for Python 3.3 and 3.5
// mod cellobject; TODO excluded by PEP-384
mod iterobject; // TODO supports PEP-384 only; needs adjustment for Python 3.3 and 3.5
// mod genobject; TODO excluded by PEP-384
mod descrobject; // TODO supports PEP-384 only; needs adjustment for Python 3.3 and 3.5
mod warnings; // TODO supports PEP-384 only; needs adjustment for Python 3.3 and 3.5
mod weakrefobject; // TODO supports PEP-384 only; needs adjustment for Python 3.3 and 3.5
mod structseq; // TODO supports PEP-384 only; needs adjustment for Python 3.3 and 3.5
// mod namespaceobject; TODO

mod codecs; // TODO supports PEP-384 only; needs adjustment for Python 3.3 and 3.5
mod pyerrors; // TODO supports PEP-384 only; needs adjustment for Python 3.3 and 3.5

mod pystate; // TODO supports PEP-384 only; needs adjustment for Python 3.3 and 3.5

#[cfg(Py_LIMITED_API)] mod pyarena {}
#[cfg(not(Py_LIMITED_API))] mod pyarena; // TODO: incomplete
mod modsupport; // TODO supports PEP-384 only; needs adjustment for Python 3.3 and 3.5
mod pythonrun; // TODO some functions need to be moved to pylifecycle
//mod pylifecycle; // TODO new in 3.5
mod ceval; // TODO supports PEP-384 only; needs adjustment for Python 3.3 and 3.5
mod sysmodule; // TODO supports PEP-384 only; needs adjustment for Python 3.3 and 3.5
mod intrcheck; // TODO supports PEP-384 only; needs adjustment for Python 3.3 and 3.5
mod import; // TODO supports PEP-384 only; needs adjustment for Python 3.3 and 3.5

mod objectabstract; // TODO supports PEP-384 only; needs adjustment for Python 3.3 and 3.5
mod bltinmodule; // TODO supports PEP-384 only; needs adjustment for Python 3.3 and 3.5

#[cfg(Py_LIMITED_API)] mod code {}
#[cfg(not(Py_LIMITED_API))] mod code;

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

pub enum PyFrameObject {}

