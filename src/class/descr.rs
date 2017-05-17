// Copyright (c) 2017-present PyO3 Project and Contributors

//! Python Description Interface
//!
//! more information
//! https://docs.python.org/3/reference/datamodel.html#implementing-descriptors

use std::os::raw::c_int;

use ffi;
use err::{PyErr, PyResult};
use python::{Python, PythonObject};
use objects::{exc, PyObject};
use class::{NO_METHODS, NO_PY_METHODS};
use callback::{PyObjectCallbackConverter, UnitCallbackConverter};

/// Descriptor interface
pub trait PyDescrProtocol {

    fn __get__(&self, py: Python, instance: &PyObject, owner: &PyObject) -> PyResult<PyObject>;

    fn __set__(&self, py: Python, instance: &PyObject, value: &PyObject) -> PyResult<()>;

    fn __delete__(&self, py: Python, instance: &PyObject) -> PyResult<()>;

    fn __set_name__(&self, py: Python, instance: &PyObject) -> PyResult<()>;

}

impl<P> PyDescrProtocol for P {
    default fn __get__(&self, py: Python, _: &PyObject, _: &PyObject) -> PyResult<PyObject> {
        Err(PyErr::new::<exc::NotImplementedError, _>(py, "Not implemented"))
    }

    default fn __set__(&self, py: Python, _: &PyObject, _: &PyObject) -> PyResult<()> {
        Err(PyErr::new::<exc::NotImplementedError, _>(py, "Not implemented"))
    }

    default fn __delete__(&self, py: Python, _: &PyObject) -> PyResult<()> {
        Err(PyErr::new::<exc::NotImplementedError, _>(py, "Not implemented"))
    }

    default fn __set_name__(&self, py: Python, _: &PyObject) -> PyResult<()> {
        Err(PyErr::new::<exc::NotImplementedError, _>(py, "Not implemented"))
    }
}


pub fn get_descrfunc<T>() -> Option<ffi::descrgetfunc>
    where T: PyDescrProtocol + PyDescrProtocolImpl + PythonObject
{
    if T::methods().contains(&"__get__") {
        py_ternary_func!(PyDescrProtocol, T::__get__, PyObjectCallbackConverter)
    } else {
        None
    }
}

pub fn set_descrfunc<T>() -> Option<ffi::descrsetfunc>
    where T: PyDescrProtocol + PyDescrProtocolImpl + PythonObject
{
    if T::methods().contains(&"__set__") {
        py_ternary_func!(PyDescrProtocol, T::__set__, UnitCallbackConverter, c_int)
    } else {
        None
    }
}

#[doc(hidden)]
pub trait PyDescrProtocolImpl {
    fn methods() -> &'static [&'static str];

    fn py_methods() -> &'static [::methods::PyMethodDefType];
}

impl<T> PyDescrProtocolImpl for T {
    default fn methods() -> &'static [&'static str] {
        NO_METHODS
    }
    default fn py_methods() -> &'static [::methods::PyMethodDefType] {
        NO_PY_METHODS
    }
}
