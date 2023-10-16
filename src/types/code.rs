use crate::ffi;
use crate::PyAny;

/// Represents a Python code object.
#[repr(transparent)]
pub struct PyCode(PyAny);

pyobject_native_type_core!(
    PyCode,
    pyobject_native_static_type_object!(ffi::PyCode_Type),
    #checkfunction=ffi::PyCode_Check
);
