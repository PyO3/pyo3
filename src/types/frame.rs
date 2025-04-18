use crate::ffi;
use crate::PyAny;

/// Represents a Python frame.
///
/// Values of this type are accessed via PyO3's smart pointers, e.g. as
/// [`Py<PyFrame>`][crate::Py] or [`Bound<'py, PyFrame>`][crate::Bound].
#[repr(transparent)]
pub struct PyFrame(PyAny);

pyobject_native_type_core!(PyFrame, #checkfunction=ffi::PyFrame_Check);
pyobject_native_type_object_methods!(PyFrame, #global=ffi::PyFrame_Type);
