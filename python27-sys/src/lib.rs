#![allow(raw_pointer_derive, non_camel_case_types, non_upper_case_globals, non_snake_case)]

extern crate libc;

pub use pyport::*;
pub use pymem::*;
pub use object::*;
pub use objimpl::*;
pub use pydebug::*;
#[cfg(feature="Py_USING_UNICODE")]
pub use unicodeobject::*;
pub use intobject::*;
pub use boolobject::*;
pub use longobject::*;
pub use floatobject::*;
pub use complexobject::*;
pub use rangeobject::*;
pub use memoryobject::*;
pub use bufferobject::*;
pub use stringobject::*;
pub use bytesobject::*;
pub use bytearrayobject::*;
pub use tupleobject::*;
pub use listobject::*;
pub use dictobject::*;
pub use enumobject::*;
pub use setobject::*;
pub use pyerrors::*;
pub use pystate::*;
pub use pystate::PyGILState_STATE::*;
pub use methodobject::*;
pub use moduleobject::*;
pub use funcobject::*;
pub use classobject::*;
pub use descrobject::*;
pub use warnings::*;
pub use pyarena::*;
pub use modsupport::*;
pub use pythonrun::*;
pub use ceval::*;
pub use import::*;
pub use objectabstract::*;
pub use code::*;
pub use eval::*;
pub use structmember::PyMemberDef;

mod pyport;
mod pymem;
mod object;
mod objimpl;
mod pydebug;
#[cfg(feature="Py_USING_UNICODE")]
mod unicodeobject; // TODO: incomplete
mod intobject;
mod boolobject;
mod longobject;
mod floatobject;
mod complexobject;
mod rangeobject;
mod stringobject;
mod memoryobject;
mod bufferobject;
mod bytesobject;
mod bytearrayobject;
mod tupleobject;
mod listobject;
mod dictobject;
mod enumobject;
mod setobject;
mod methodobject;
mod moduleobject;
mod funcobject;
mod classobject;
// mod fileobject; // TODO: incomplete
// mod cobject; // TODO: incomplete
// mod pycapsule; // TODO: incomplete
// mod traceback; // TODO: incomplete
// mod sliceobject; // TODO: incomplete
// mod cellobject; // TODO: incomplete
// mod iterobject; // TODO: incomplete
// mod genobject; // TODO: incomplete
mod descrobject; // TODO: incomplete
mod warnings;
// mod weakrefobject; // TODO: incomplete

// mod codecs; // TODO: incomplete
mod pyerrors;

mod pystate;

mod pyarena;
mod modsupport;
mod pythonrun;
mod ceval;
// mod sysmodule; // TODO: incomplete
// mod intrcheck; // TODO: incomplete
mod import;

mod objectabstract;

mod code;
// mod compile; // TODO: incomplete
mod eval;

// mod pyctype; // TODO: incomplete
// mod pystrtod; // TODO: incomplete
// mod pystrcmp; // TODO: incomplete
// mod dtoa; // TODO: incomplete

// mod pyfpe; // TODO: incomplete

// Additional headers that are not exported by Python.h
pub mod structmember;


#[cfg(not(feature="Py_USING_UNICODE"))]
#[inline(always)]
pub fn PyUnicode_Check(op : *mut PyObject) -> c_int { 0 }

#[cfg(not(feature="Py_USING_UNICODE"))]
#[inline(always)]
pub fn PyUnicode_CheckExact(op : *mut PyObject) -> c_int { 0 }


