use crate::ffi;
use crate::prelude::*;

/// Represents a builtin Python function object.
#[repr(transparent)]
pub struct PyCFunction(PyAny);

pyobject_native_var_type!(PyCFunction, ffi::PyCFunction_Type, ffi::PyCFunction_Check);

/// Represents a Python function object.
#[repr(transparent)]
pub struct PyFunction(PyAny);

pyobject_native_var_type!(PyFunction, ffi::PyFunction_Type, ffi::PyFunction_Check);
