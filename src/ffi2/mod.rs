//! Rust FFI declarations for Python 2
#![no_std]
#![allow(non_camel_case_types, non_upper_case_globals, non_snake_case)]
#![cfg_attr(feature="cargo-clippy", allow(inline_always))]

use std::os::raw::c_int;

pub use self::pyport::*;
pub use self::pymem::*;
pub use self::object::*;
pub use self::objimpl::*;
pub use self::pydebug::*;
#[cfg(py_sys_config="Py_USING_UNICODE")]
pub use self::unicodeobject::*;
pub use self::intobject::*;
pub use self::boolobject::*;
pub use self::longobject::*;
pub use self::floatobject::*;
pub use self::complexobject::*;
pub use self::rangeobject::*;
pub use self::memoryobject::*;
pub use self::bufferobject::*;
pub use self::stringobject::*;
pub use self::bytesobject::*;
pub use self::bytearrayobject::*;
pub use self::tupleobject::*;
pub use self::listobject::*;
pub use self::dictobject::*;
pub use self::enumobject::*;
pub use self::setobject::*;
pub use self::pyerrors::*;
pub use self::pystate::*;
pub use self::pystate::PyGILState_STATE::*;
pub use self::methodobject::*;
pub use self::moduleobject::*;
pub use self::funcobject::*;
pub use self::classobject::*;
pub use self::fileobject::*;
pub use self::cobject::*;
pub use self::pycapsule::*;
pub use self::traceback::*;
pub use self::sliceobject::*;
pub use self::cellobject::*;
pub use self::iterobject::*;
pub use self::genobject::*;
pub use self::descrobject::*;
pub use self::warnings::*;
pub use self::weakrefobject::*;
pub use self::pyarena::*;
pub use self::modsupport::*;
pub use self::pythonrun::*;
pub use self::ceval::*;
pub use self::import::*;
pub use self::objectabstract::*;
pub use self::code::*;
pub use self::compile::*;
pub use self::eval::*;
pub use self::structmember::PyMemberDef;
pub use self::frameobject::PyFrameObject;

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

pub const Py_single_input: c_int = 256;
pub const Py_file_input: c_int = 257;
pub const Py_eval_input: c_int = 258;

#[cfg(not(py_sys_config="Py_USING_UNICODE"))]
#[inline(always)]
pub fn PyUnicode_Check(op : *mut PyObject) -> libc::c_int { 0 }

#[cfg(not(py_sys_config="Py_USING_UNICODE"))]
#[inline(always)]
pub fn PyUnicode_CheckExact(op : *mut PyObject) -> libc::c_int { 0 }
