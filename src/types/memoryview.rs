use crate::err::PyResult;
use crate::{ffi, AsPyPointer, PyAny};

/// Represents a Python `memoryview`.
#[repr(transparent)]
pub struct PyMemoryView(PyAny);

pyobject_native_type_core!(PyMemoryView, pyobject_native_static_type_object!(ffi::PyMemoryView_Type), #checkfunction=ffi::PyMemoryView_Check);

impl PyMemoryView {
    /// Creates a new Python `memoryview` object from another Python object that
    /// implements the buffer protocol.
    pub fn from(src: &PyAny) -> PyResult<&PyMemoryView> {
        unsafe {
            src.py()
                .from_owned_ptr_or_err(ffi::PyMemoryView_FromObject(src.as_ptr()))
        }
    }
}
