#![no_std]
#![allow(non_camel_case_types, non_snake_case, non_upper_case_globals)]
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
#[cfg(Py_3_4)] pub use pyhash::*;

pub use pydebug::*;

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
#[cfg(Py_3_6)] pub use osmodule::*;
pub use intrcheck::*;
pub use import::*;

pub use objectabstract::*;
pub use bltinmodule::*;

pub use code::*;
pub use compile::*;
pub use eval::*;

pub use pystrtod::*;
pub use frameobject::PyFrameObject;

mod pyport;
// mod pymacro; contains nothing of interest for Rust

// mod pyatomic; contains nothing of interest for Rust

// mod pymath; contains nothing of interest for Rust

// [cfg(not(Py_LIMITED_API))]
// mod pytime; contains nothing of interest

mod pymem;

mod object;
mod objimpl;
mod typeslots;
#[cfg(Py_3_4)] mod pyhash;

mod pydebug;

mod bytearrayobject;
mod bytesobject;
mod unicodeobject;
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
#[cfg(Py_3_6)] mod osmodule;
mod intrcheck; // TODO supports PEP-384 only; needs adjustment for Python 3.3 and 3.5
mod import; // TODO supports PEP-384 only; needs adjustment for Python 3.3 and 3.5

mod objectabstract;
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

#[cfg(not(Py_LIMITED_API))]
pub mod frameobject;
#[cfg(Py_LIMITED_API)]
pub mod frameobject {
    pub enum PyFrameObject {}
}

