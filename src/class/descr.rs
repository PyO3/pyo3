// Copyright (c) 2017-present PyO3 Project and Contributors

//! Python Description Interface
//!
//! more information
//! https://docs.python.org/3/reference/datamodel.html#implementing-descriptors

use std::os::raw::c_int;

use ffi;
use err::PyResult;
use python::{Python, PythonObject};
use objects::{PyObject, PyType};
use callback::{PyObjectCallbackConverter, UnitCallbackConverter};
use class::methods::PyMethodDef;
use ::{ToPyObject, FromPyObject};

/// Descriptor interface
#[allow(unused_variables)]
pub trait PyDescrProtocol: PythonObject {

    fn __get__(&self, py: Python, instance: &PyObject, owner: Option<PyType>)
               -> Self::Result where Self: PyDescrGetProtocol { unimplemented!() }

    fn __set__(&self, py: Python, instance: &PyObject, value: &PyObject)
               -> Self::Result where Self: PyDescrSetProtocol { unimplemented!() }

    fn __delete__(&self, py: Python, instance: &PyObject)
                  -> Self::Result where Self: PyDescrDeleteProtocol { unimplemented!() }

    fn __set_name__(&self, py: Python, instance: &PyObject)
                    -> Self::Result where Self: PyDescrSetNameProtocol { unimplemented!() }
}

pub trait PyDescrGetProtocol: PyDescrProtocol {
    type Inst: for<'a> FromPyObject<'a>;
    type Owner: for<'a> FromPyObject<'a>;
    type Success: ToPyObject;
    type Result: Into<PyResult<Self::Success>>;
}

pub trait PyDescrSetProtocol: PyDescrProtocol {
    type Inst: for<'a> FromPyObject<'a>;
    type Value: for<'a> FromPyObject<'a>;
    type Result: Into<PyResult<()>>;
}

pub trait PyDescrDeleteProtocol: PyDescrProtocol {
    type Inst: for<'a> FromPyObject<'a>;
    type Result: Into<PyResult<()>>;
}

pub trait PyDescrSetNameProtocol: PyDescrProtocol {
    type Inst: for<'a> FromPyObject<'a>;
    type Result: Into<PyResult<()>>;
}


pub trait PyDescrGetProtocolImpl {
    fn tp_descr_get() -> Option<ffi::descrgetfunc>;
}
impl<T> PyDescrGetProtocolImpl for T where T: PyDescrProtocol {
    default fn tp_descr_get() -> Option<ffi::descrgetfunc> {
        None
    }
}
impl<T> PyDescrGetProtocolImpl for T where T: PyDescrGetProtocol
{
    fn tp_descr_get() -> Option<ffi::descrgetfunc> {
        py_ternary_func!(PyDescrGetProtocol, T::__get__, PyObjectCallbackConverter)
    }
}
pub trait PyDescrSetProtocolImpl {
    fn tp_descr_set() -> Option<ffi::descrsetfunc>;
}
impl<T> PyDescrSetProtocolImpl for T where T: PyDescrProtocol {
    default fn tp_descr_set() -> Option<ffi::descrsetfunc> {
        None
    }
}
impl<T> PyDescrSetProtocolImpl for T where T: PyDescrSetProtocol
{
    fn tp_descr_set() -> Option<ffi::descrsetfunc> {
        py_ternary_func!(PyDescrSetProtocol, T::__set__, UnitCallbackConverter, c_int)
    }
}

pub trait PyDescrDelProtocolImpl {
    fn __del__() -> Option<PyMethodDef>;
}
impl<T> PyDescrDelProtocolImpl for T where T: PyDescrProtocol {
    default fn __del__() -> Option<PyMethodDef> {
        None
    }
}

pub trait PyDescrSetNameProtocolImpl {
    fn __set_name__() -> Option<PyMethodDef>;
}
impl<T> PyDescrSetNameProtocolImpl for T where T: PyDescrProtocol {
    default fn __set_name__() -> Option<PyMethodDef> {
        None
    }
}

#[doc(hidden)]
pub trait PyDescrProtocolImpl {
    fn methods() -> Vec<PyMethodDef>;
    fn tp_as_descr(type_object: &mut ffi::PyTypeObject);
}

impl<T> PyDescrProtocolImpl for T {
    default fn methods() -> Vec<PyMethodDef> {
        Vec::new()
    }
    default fn tp_as_descr(_type_object: &mut ffi::PyTypeObject) {
    }
}

impl<T> PyDescrProtocolImpl for T where T: PyDescrProtocol {
    fn methods() -> Vec<PyMethodDef> {
        Vec::new()
    }
    fn tp_as_descr(type_object: &mut ffi::PyTypeObject) {
        type_object.tp_descr_get = Self::tp_descr_get();
        type_object.tp_descr_set = Self::tp_descr_set();
    }
}
