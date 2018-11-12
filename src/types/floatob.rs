// Copyright (c) 2017-present PyO3 Project and Contributors
//
// based on Daniel Grunwald's https://github.com/dgrunwald/rust-cpython

use crate::conversion::{IntoPyObject, ToPyObject};
use crate::err::PyErr;
use crate::ffi;
use crate::instance::{Py, PyObjectWithGIL};
use crate::object::PyObject;
use crate::objectprotocol::ObjectProtocol;
use crate::python::{Python, ToPyPointer};
use crate::types::PyObjectRef;
use crate::FromPyObject;
use crate::PyResult;
use std::os::raw::c_double;

/// Represents a Python `float` object.
///
/// You can usually avoid directly working with this type
/// by using [`ToPyObject`](trait.ToPyObject.html)
/// and [extract](struct.PyObject.html#method.extract)
/// with `f32`/`f64`.
#[repr(transparent)]
pub struct PyFloat(PyObject);

pyobject_native_type!(PyFloat, ffi::PyFloat_Type, ffi::PyFloat_Check);

impl PyFloat {
    /// Creates a new Python `float` object.
    pub fn new(_py: Python, val: c_double) -> Py<PyFloat> {
        unsafe { Py::from_owned_ptr_or_panic(ffi::PyFloat_FromDouble(val)) }
    }

    /// Gets the value of this float.
    pub fn value(&self) -> c_double {
        unsafe { ffi::PyFloat_AsDouble(self.0.as_ptr()) }
    }
}

impl ToPyObject for f64 {
    fn to_object(&self, py: Python) -> PyObject {
        PyFloat::new(py, *self).into()
    }
}

impl IntoPyObject for f64 {
    fn into_object(self, py: Python) -> PyObject {
        PyFloat::new(py, self).into()
    }
}

impl<'source> FromPyObject<'source> for f64 {
    // PyFloat_AsDouble returns -1.0 upon failure
    #![cfg_attr(feature = "cargo-clippy", allow(float_cmp))]
    fn extract(obj: &'source PyObjectRef) -> PyResult<Self> {
        let v = unsafe { ffi::PyFloat_AsDouble(obj.as_ptr()) };

        if v == -1.0 && PyErr::occurred(obj.py()) {
            Err(PyErr::fetch(obj.py()))
        } else {
            Ok(v)
        }
    }
}

impl ToPyObject for f32 {
    fn to_object(&self, py: Python) -> PyObject {
        PyFloat::new(py, f64::from(*self)).into()
    }
}

impl IntoPyObject for f32 {
    fn into_object(self, py: Python) -> PyObject {
        PyFloat::new(py, f64::from(self)).into()
    }
}

impl<'source> FromPyObject<'source> for f32 {
    fn extract(obj: &'source PyObjectRef) -> PyResult<Self> {
        Ok(obj.extract::<f64>()? as f32)
    }
}

#[cfg(test)]
mod test {
    use crate::conversion::ToPyObject;
    use crate::python::Python;

    macro_rules! num_to_py_object_and_back (
        ($func_name:ident, $t1:ty, $t2:ty) => (
            #[test]
            fn $func_name() {
                let gil = Python::acquire_gil();
                let py = gil.python();
                let val = 123 as $t1;
                let obj = val.to_object(py);
                assert_eq!(obj.extract::<$t2>(py).unwrap(), val as $t2);
            }
        )
    );

    num_to_py_object_and_back!(to_from_f64, f64, f64);
    num_to_py_object_and_back!(to_from_f32, f32, f32);
    num_to_py_object_and_back!(int_to_float, i32, f64);
}
