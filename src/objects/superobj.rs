// Copyright (c) 2017-present PyO3 Project and Contributors

use std;

use ffi;
use object::PyObject;
use objects::{PyObjectRef, PyDict};
use objectprotocol::ObjectProtocol;
use conversion::IntoPyTuple;
use python::{ToPyPointer, IntoPyPointer};
use err::{self, PyResult};
use instance::PyObjectWithToken;

/// Represents a reference to a Python `super object`.
pub struct PySuper(PyObject);

pyobject_convert!(PySuper);
pyobject_nativetype!(PySuper, PySuper_Type, PySuper_Check);


impl PySuper {

    pub fn new<T>(object: &T) -> &PySuper where T: PyObjectWithToken + ToPyPointer {
        // Create the arguments for super()
        unsafe {
            let o = object.py().from_borrowed_ptr(object.as_ptr());
            let args = (object.get_type(), o).into_tuple(object.py()).into_ptr();

            // Creat the class super()
            let ptr = ffi::PyType_GenericNew(&mut ffi::PySuper_Type, args, std::ptr::null_mut());
            let oref = object.py().cast_from_ptr(ptr);

            // call __init__ on super object
            if (*ffi::Py_TYPE(ptr)).tp_init.unwrap()(ptr, args, std::ptr::null_mut()) == -1 {
                err::panic_after_error()
            }
            oref
        }
    }

    pub fn __new__<A>(&self, args: A, kwargs: Option<&PyDict>)
                      -> PyResult<&PyObjectRef>
        where A: IntoPyTuple
    {
        self.call_method("__new__", args, kwargs)
    }

    pub fn __init__<A>(&self, args: A, kwargs: Option<&PyDict>)
                       -> PyResult<&PyObjectRef>
        where A: IntoPyTuple
    {
        self.call_method("__init__", args, kwargs)
    }
}
