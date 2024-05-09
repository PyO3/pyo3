use crate::err::PyResult;
use crate::ffi_ptr_ext::FfiPtrExt;
use crate::py_result_ext::PyResultExt;
#[cfg(feature = "gil-refs")]
use crate::PyNativeType;
use crate::{ffi, AsPyPointer, Bound, PyAny};

/// Represents a Python `memoryview`.
#[repr(transparent)]
pub struct PyMemoryView(PyAny);

pyobject_native_type_core!(PyMemoryView, pyobject_native_static_type_object!(ffi::PyMemoryView_Type), #checkfunction=ffi::PyMemoryView_Check);

impl PyMemoryView {
    /// Deprecated form of [`PyMemoryView::from_bound`]
    #[cfg(feature = "gil-refs")]
    #[deprecated(
        since = "0.21.0",
        note = "`PyMemoryView::from` will be replaced by `PyMemoryView::from_bound` in a future PyO3 version"
    )]
    pub fn from(src: &PyAny) -> PyResult<&PyMemoryView> {
        PyMemoryView::from_bound(&src.as_borrowed()).map(Bound::into_gil_ref)
    }

    /// Creates a new Python `memoryview` object from another Python object that
    /// implements the buffer protocol.
    pub fn from_bound<'py>(src: &Bound<'py, PyAny>) -> PyResult<Bound<'py, Self>> {
        unsafe {
            ffi::PyMemoryView_FromObject(src.as_ptr())
                .assume_owned_or_err(src.py())
                .downcast_into_unchecked()
        }
    }
}

#[cfg(feature = "gil-refs")]
impl<'py> TryFrom<&'py PyAny> for &'py PyMemoryView {
    type Error = crate::PyErr;

    /// Creates a new Python `memoryview` object from another Python object that
    /// implements the buffer protocol.
    fn try_from(value: &'py PyAny) -> Result<Self, Self::Error> {
        PyMemoryView::from_bound(&value.as_borrowed()).map(Bound::into_gil_ref)
    }
}

impl<'py> TryFrom<&Bound<'py, PyAny>> for Bound<'py, PyMemoryView> {
    type Error = crate::PyErr;

    /// Creates a new Python `memoryview` object from another Python object that
    /// implements the buffer protocol.
    fn try_from(value: &Bound<'py, PyAny>) -> Result<Self, Self::Error> {
        PyMemoryView::from_bound(value)
    }
}
