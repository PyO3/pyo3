#![no_std]
#![allow(non_camel_case_types, non_upper_case_globals, non_snake_case)]

extern crate libc;

pub use pyport::*;
pub use pymem::*;
pub use object::*;
pub use objimpl::*;
pub use pydebug::*;
#[cfg(py_sys_config="Py_USING_UNICODE")]
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
pub use fileobject::*;
pub use cobject::*;
pub use pycapsule::*;
pub use traceback::*;
pub use sliceobject::*;
pub use cellobject::*;
pub use iterobject::*;
pub use genobject::*;
pub use descrobject::*;
pub use warnings::*;
pub use weakrefobject::*;
pub use pyarena::*;
pub use modsupport::*;
pub use pythonrun::*;
pub use ceval::*;
pub use import::*;
pub use objectabstract::*;
pub use code::*;
pub use compile::*;
pub use eval::*;
pub use structmember::PyMemberDef;
pub use frameobject::PyFrameObject;

mod pyport;
mod pymem;
mod object;
mod objimpl;
mod pydebug;
#[cfg(py_sys_config="Py_USING_UNICODE")]
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
mod fileobject;
mod cobject;
mod pycapsule;
mod traceback;
mod sliceobject;
mod cellobject;
mod iterobject;
mod genobject;
mod descrobject;
mod warnings;
mod weakrefobject;

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
mod compile;
mod eval;

// mod pyctype; // TODO: incomplete
// mod pystrtod; // TODO: incomplete
// mod pystrcmp; // TODO: incomplete
// mod dtoa; // TODO: incomplete

// mod pyfpe; // TODO: incomplete

// Additional headers that are not exported by Python.h
pub mod structmember;
pub mod frameobject;

pub const Py_single_input: libc::c_int = 256;
pub const Py_file_input: libc::c_int = 257;
pub const Py_eval_input: libc::c_int = 258;

#[cfg(not(py_sys_config="Py_USING_UNICODE"))]
#[inline(always)]
pub fn PyUnicode_Check(op : *mut PyObject) -> libc::c_int { 0 }

#[cfg(not(py_sys_config="Py_USING_UNICODE"))]
#[inline(always)]
pub fn PyUnicode_CheckExact(op : *mut PyObject) -> libc::c_int { 0 }


