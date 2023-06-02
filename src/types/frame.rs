use crate::ffi;
use crate::PyAny;

/// Represents a Python frame.
#[repr(transparent)]
pub struct PyFrame(PyAny);

pyobject_native_type_core!(
    PyFrame,
    ffi::PyFrame_Type,
    #checkfunction=ffi::PyFrame_Check
);
