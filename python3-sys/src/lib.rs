#![allow(non_camel_case_types, non_snake_case, non_upper_case_globals, raw_pointer_derive)]

// Based on the headers of Python 3.4.3
// Supports the stable ABI (PEP 384) only.

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

pub use modsupport::*;
pub use pythonrun::*;
pub use ceval::*;
pub use sysmodule::*;
pub use intrcheck::*;
pub use import::*;

pub use objectabstract::*;
pub use bltinmodule::*;

pub use eval::*;

pub use pystrtod::*;

mod pyport;
// mod pymacro; contains nothing of interest for Rust

// mod pyatomic; excluded by PEP-384

// mod pymath; contains nothing of interest for Rust
// mod pytime; excluded by PEP-384
mod pymem;

mod object;
mod objimpl;
mod typeslots;
// mod pyhash; contains nothing of interest

// mod pydebug; excluded by PEP-384

mod bytearrayobject;
mod bytesobject;
mod unicodeobject;
mod longobject;
// mod longintrepr; excluded by PEP-384
mod boolobject;
mod floatobject;
mod complexobject;
mod rangeobject;
mod memoryobject;
mod tupleobject;
mod listobject;
mod dictobject;
mod enumobject;
mod setobject;
mod methodobject;
mod moduleobject;
// mod funcobject; excluded by PEP-384
// mod classobject; excluded by PEP-384
mod fileobject;
mod pycapsule;
mod traceback;
mod sliceobject;
// mod cellobject; excluded by PEP-384
mod iterobject;
// mod genobject; excluded by PEP-384
mod descrobject;
mod warnings;
mod weakrefobject;
mod structseq;
// mod namespaceobject; contains nothing of interest

mod codecs;
mod pyerrors;

mod pystate;

// mod pyarena; excluded by PEP-384
mod modsupport;
mod pythonrun;
mod ceval;
mod sysmodule;
mod intrcheck;
mod import;

mod objectabstract;
mod bltinmodule;

// mod compile; excluded by PEP-384
mod eval;

// mod pyctype; excluded by PEP-384
mod pystrtod;
// mod pystrcmp; nothing interesting for Rust
// mod dtoa; excluded by PEP-384
// mod fileutils; no public functions
// mod pyfpe; probably not interesting for rust

// Additional headers that are not exported by Python.h
pub mod structmember;

pub enum PyFrameObject {}

