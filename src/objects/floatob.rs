// Copyright (c) 2017-present PyO3 Project and Contributors
//
// based on Daniel Grunwald's https://github.com/dgrunwald/rust-cpython

use std::os::raw::c_double;

use ffi;
use objects::PyObject;
use pointers::PyPtr;
use python::{ToPyPointer, Python};
use err::PyErr;
use conversion::{ToPyObject, IntoPyObject};

/// Represents a Python `float` object.
///
/// You can usually avoid directly working with this type
/// by using [`ToPyObject`](trait.ToPyObject.html)
/// and [extract](struct.PyObject.html#method.extract)
/// with `f32`/`f64`.
pub struct PyFloat(PyPtr);

pyobject_convert!(PyFloat);
pyobject_nativetype!(PyFloat, PyFloat_Check, PyFloat_Type);


impl PyFloat {
    /// Creates a new Python `float` object.
    pub fn new(_py: Python, val: c_double) -> PyFloat {
        unsafe {
            PyFloat(PyPtr::from_owned_ptr_or_panic(ffi::PyFloat_FromDouble(val)))
        }
    }

    /// Gets the value of this float.
    pub fn value(&self, _py: Python) -> c_double {
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

pyobject_extract!(py, obj to f64 => {
    let v = unsafe { ffi::PyFloat_AsDouble(obj.as_ptr()) };
    if v == -1.0 && PyErr::occurred(py) {
        Err(PyErr::fetch(py))
    } else {
        Ok(v)
    }
});

impl ToPyObject for f32 {
    fn to_object(&self, py: Python) -> PyObject {
        PyFloat::new(py, *self as f64).into()
    }
}
impl IntoPyObject for f32 {
    fn into_object(self, py: Python) -> PyObject {
        PyFloat::new(py, self as f64).into()
    }
}

pyobject_extract!(py, obj to f32 => {
    Ok(try!(obj.extract::<f64>(py)) as f32)
});


#[cfg(test)]
mod test {
    use python::Python;
    use conversion::ToPyObject;

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
    num_to_py_object_and_back!(float_to_i32, f64, i32);
    num_to_py_object_and_back!(float_to_u32, f64, u32);
    num_to_py_object_and_back!(float_to_i64, f64, i64);
    num_to_py_object_and_back!(float_to_u64, f64, u64);
    num_to_py_object_and_back!(int_to_float, i32, f64);
}
