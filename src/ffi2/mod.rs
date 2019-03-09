//! Rust FFI declarations for Python 2
#![allow(non_camel_case_types, non_upper_case_globals, non_snake_case)]

use std::os::raw::c_int;

pub use self::boolobject::*;
pub use self::bufferobject::*;
pub use self::bytearrayobject::*;
pub use self::bytesobject::*;
pub use self::cellobject::*;
pub use self::ceval::*;
pub use self::classobject::*;
pub use self::cobject::*;
pub use self::code::*;
pub use self::compile::*;
pub use self::complexobject::*;
pub use self::descrobject::*;
pub use self::dictobject::*;
pub use self::enumobject::*;
pub use self::eval::*;
pub use self::fileobject::*;
pub use self::floatobject::*;
pub use self::frameobject::PyFrameObject;
pub use self::funcobject::*;
pub use self::genobject::*;
pub use self::import::*;
pub use self::intobject::*;
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
pub use self::pyarena::*;
pub use self::pycapsule::*;
pub use self::pydebug::*;
pub use self::pyerrors::*;
pub use self::pymem::*;
pub use self::pyport::*;
pub use self::pystate::PyGILState_STATE::*;
pub use self::pystate::*;
pub use self::pythonrun::*;
pub use self::rangeobject::*;
pub use self::setobject::*;
pub use self::sliceobject::*;
pub use self::stringobject::*;
pub use self::structmember::PyMemberDef;
pub use self::traceback::*;
pub use self::tupleobject::*;
#[cfg(py_sys_config = "Py_USING_UNICODE")]
pub use self::unicodeobject::*;
pub use self::warnings::*;
pub use self::weakrefobject::*;

mod boolobject;
mod bufferobject;
mod bytearrayobject;
mod bytesobject;
mod cellobject;
mod classobject;
mod cobject;
mod complexobject;
mod descrobject;
mod dictobject;
mod enumobject;
mod fileobject;
mod floatobject;
mod funcobject;
mod genobject;
mod intobject;
mod iterobject;
mod listobject;
mod longobject;
mod memoryobject;
mod methodobject;
mod moduleobject;
mod object;
mod objimpl;
mod pycapsule;
mod pydebug;
mod pymem;
mod pyport;
mod rangeobject;
mod setobject;
mod sliceobject;
mod stringobject;
mod traceback;
mod tupleobject;
#[cfg(py_sys_config = "Py_USING_UNICODE")]
mod unicodeobject; // TODO: incomplete
mod warnings;
mod weakrefobject;

// mod codecs; // TODO: incomplete
mod pyerrors;

mod pystate;

mod ceval;
mod modsupport;
mod pyarena;
mod pythonrun;
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
pub mod frameobject;
pub mod structmember;

pub const Py_single_input: c_int = 256;
pub const Py_file_input: c_int = 257;
pub const Py_eval_input: c_int = 258;

#[cfg(not(py_sys_config = "Py_USING_UNICODE"))]
#[inline]
pub fn PyUnicode_Check(op: *mut PyObject) -> libc::c_int {
    0
}

#[cfg(not(py_sys_config = "Py_USING_UNICODE"))]
#[inline]
pub fn PyUnicode_CheckExact(op: *mut PyObject) -> libc::c_int {
    0
}
