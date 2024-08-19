use crate::err::PyResult;
use crate::ffi_ptr_ext::FfiPtrExt;
use crate::py_result_ext::PyResultExt;
use crate::{ffi, Bound, PyAny};

/// Represents a Python `memoryview`.
///
/// Values of this type are accessed via PyO3's smart pointers, e.g. as
/// [`Py<PyMemoryView>`][crate::Py] or [`Bound<'py, PyMemoryView>`][Bound].
#[repr(transparent)]
pub struct PyMemoryView(PyAny);

pyobject_native_type_core!(PyMemoryView, pyobject_native_static_type_object!(ffi::PyMemoryView_Type), #checkfunction=ffi::PyMemoryView_Check);

impl PyMemoryView {
    /// Creates a new Python `memoryview` object from another Python object that
    /// implements the buffer protocol.
    pub fn from<'py>(src: &Bound<'py, PyAny>) -> PyResult<Bound<'py, Self>> {
        unsafe {
            ffi::PyMemoryView_FromObject(src.as_ptr())
                .assume_owned_or_err(src.py())
                .downcast_into_unchecked()
        }
    }

    /// Deprecated name for [`PyMemoryView::from`].
    #[deprecated(since = "0.23.0", note = "renamed to `PyMemoryView::from`")]
    #[inline]
    pub fn from_bound<'py>(src: &Bound<'py, PyAny>) -> PyResult<Bound<'py, Self>> {
        Self::from(src)
    }
}

impl<'py> TryFrom<&Bound<'py, PyAny>> for Bound<'py, PyMemoryView> {
    type Error = crate::PyErr;

    /// Creates a new Python `memoryview` object from another Python object that
    /// implements the buffer protocol.
    fn try_from(value: &Bound<'py, PyAny>) -> Result<Self, Self::Error> {
        PyMemoryView::from(value)
    }
}
