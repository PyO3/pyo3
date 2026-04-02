use crate::err::PyResult;
use crate::ffi_ptr_ext::FfiPtrExt;
use crate::py_result_ext::PyResultExt;
#[cfg(any(Py_3_11, not(Py_LIMITED_API)))]
use crate::Python;
use crate::{ffi, Bound, PyAny};

/// Represents a Python `memoryview`.
///
/// Values of this type are accessed via PyO3's smart pointers, e.g. as
/// [`Py<PyMemoryView>`][crate::Py] or [`Bound<'py, PyMemoryView>`][Bound].
#[repr(transparent)]
pub struct PyMemoryView(PyAny);

pyobject_native_type_core!(PyMemoryView, pyobject_native_static_type_object!(ffi::PyMemoryView_Type), "builtins", "memoryview", #checkfunction=ffi::PyMemoryView_Check);

impl PyMemoryView {
    /// Creates a new Python `memoryview` object from another Python object that
    /// implements the buffer protocol.
    pub fn from<'py>(src: &Bound<'py, PyAny>) -> PyResult<Bound<'py, Self>> {
        unsafe {
            ffi::PyMemoryView_FromObject(src.as_ptr())
                .assume_owned_or_err(src.py())
                .cast_into_unchecked()
        }
    }

    /// Creates a new Python `memoryview` that exposes a read-only view of the
    /// byte data owned by a frozen `PyClass` instance.
    ///
    /// This avoids copying data when you want to expose the internal buffer of
    /// a Python object as a `memoryview`. The `owner` keeps the data alive for
    /// the lifetime of the `memoryview`.
    ///
    /// # Arguments
    ///
    /// * `py` — The Python GIL token.
    /// * `owner` — A `Py<T>` reference to a frozen `PyClass` instance that owns
    ///   the underlying data.
    /// * `getbuf` — A closure that borrows `T` and returns the byte slice to
    ///   expose. The higher-ranked lifetime ensures the slice is derived from
    ///   `T` (or is `'static`), preventing dangling pointers.
    ///
    /// # Safety guarantees
    ///
    /// * `T: PyClass<Frozen = True>` ensures the class is immutable, so the
    ///   buffer pointer cannot be invalidated by mutation.
    /// * The `for<'a> FnOnce(&'a T) -> &'a [u8]` signature ensures the returned
    ///   slice borrows from `T` or is `'static`, preventing references to
    ///   temporaries.
    ///
    /// # Example
    ///
    /// ```rust
    /// # #[cfg(any(Py_3_11, not(Py_LIMITED_API)))]
    /// # {
    /// use pyo3::prelude::*;
    /// use pyo3::types::PyMemoryView;
    ///
    /// #[pyclass(frozen)]
    /// struct MyData {
    ///     data: Vec<u8>,
    /// }
    ///
    /// Python::attach(|py| {
    ///     let obj = Py::new(py, MyData { data: vec![1, 2, 3] }).unwrap();
    ///     let view = PyMemoryView::from_owned_buffer(py, obj, |data| &data.data).unwrap();
    ///     assert_eq!(view.len().unwrap(), 3);
    /// });
    /// # }
    /// ```
    #[cfg(any(Py_3_11, not(Py_LIMITED_API)))]
    pub fn from_owned_buffer<'py, T>(
        py: Python<'py>,
        owner: crate::Py<T>,
        getbuf: impl for<'a> FnOnce(&'a T) -> &'a [u8],
    ) -> PyResult<Bound<'py, Self>>
    where
        T: crate::PyClass<Frozen = crate::pyclass::boolean_struct::True> + Sync,
    {
        // Get the raw object pointer. This is a borrowed reference (no refcount change).
        let owner_ptr = owner.as_ptr();
        let buf = getbuf(owner.get());

        let mut view = ffi::Py_buffer::new();

        // SAFETY: PyBuffer_FillInfo initializes the Py_buffer struct. On
        // success it calls Py_INCREF on the owner object (via view.obj).
        // We pass readonly=1 since we only expose an immutable view.
        let rc = unsafe {
            ffi::PyBuffer_FillInfo(
                &mut view,
                owner_ptr,
                buf.as_ptr() as *mut std::ffi::c_void,
                buf.len() as ffi::Py_ssize_t,
                1, // readonly
                ffi::PyBUF_FULL_RO,
            )
        };
        if rc == -1 {
            return Err(crate::PyErr::fetch(py));
        }

        // SAFETY: PyMemoryView_FromBuffer creates a memoryview that takes
        // ownership of the buffer (it will call PyBuffer_Release when the
        // memoryview is deallocated, which will decref view.obj).
        unsafe {
            ffi::PyMemoryView_FromBuffer(&view)
                .assume_owned_or_err(py)
                .cast_into_unchecked()
        }
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
