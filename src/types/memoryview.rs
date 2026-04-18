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
    /// byte data owned by a frozen `PyClass` instance, without copying.
    ///
    /// `getbuf` is a closure that receives `T` borrowed from `owner` and
    /// returns the byte slice to expose. The higher-ranked lifetime ensures
    /// the slice is derived from `T` (or is `'static`), preventing dangling
    /// pointers. `T` must be a `frozen` pyclass to guarantee the byte slice
    /// cannot be mutated.
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
    ///     let obj = Bound::new(py, MyData { data: vec![1, 2, 3] }).unwrap();
    ///     let view = PyMemoryView::from_owned_buffer(&obj, |data| &data.data).unwrap();
    ///     assert_eq!(view.len().unwrap(), 3);
    /// });
    /// # }
    /// ```
    #[cfg(any(Py_3_11, not(Py_LIMITED_API)))]
    pub fn from_owned_buffer<'py, T>(
        owner: &Bound<'py, T>,
        getbuf: impl for<'a> FnOnce(&'a T) -> &'a [u8],
    ) -> PyResult<Bound<'py, Self>>
    where
        T: crate::PyClass<Frozen = crate::pyclass::boolean_struct::True> + Sync,
    {
        let py = owner.py();
        let buf = getbuf(owner.get());

        let mut view = std::mem::MaybeUninit::<ffi::Py_buffer>::uninit();

        // SAFETY: `view` points to a valid (uninitialized) `Py_buffer`.
        // `PyBuffer_FillInfo` fully initializes every field on success, and
        // increfs `owner` into `view.obj`. `owner` outlives the call because
        // it is held on the stack by `Bound`.
        let rc = unsafe {
            ffi::PyBuffer_FillInfo(
                view.as_mut_ptr(),
                owner.as_ptr(),
                buf.as_ptr() as *mut std::ffi::c_void,
                buf.len() as ffi::Py_ssize_t,
                1, // readonly
                ffi::PyBUF_FULL_RO,
            )
        };
        crate::err::error_on_minusone(py, rc)?;

        // SAFETY: `PyBuffer_FillInfo` returned success, so `view` is now
        // fully initialized.
        let view = unsafe { view.assume_init() };

        // SAFETY: `view` is a fully initialized `Py_buffer`, as required.
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

#[cfg(all(test, feature = "macros", any(Py_3_11, not(Py_LIMITED_API))))]
mod tests {
    use super::*;
    use crate::types::PyAnyMethods;
    use crate::{Bound, Python};

    #[crate::pyclass(frozen, crate = "crate")]
    struct ByteOwner {
        data: Vec<u8>,
    }

    #[test]
    fn test_from_owned_buffer_basic() {
        Python::attach(|py| {
            let owner = Bound::new(
                py,
                ByteOwner {
                    data: vec![1, 2, 3, 4, 5],
                },
            )
            .unwrap();
            let view = PyMemoryView::from_owned_buffer(&owner, |o| &o.data).unwrap();
            assert_eq!(view.len().unwrap(), 5);
            let bytes: Vec<u8> = view.call_method0("tobytes").unwrap().extract().unwrap();
            assert_eq!(bytes, vec![1, 2, 3, 4, 5]);
        });
    }

    #[test]
    fn test_from_owned_buffer_readonly() {
        Python::attach(|py| {
            let owner = Bound::new(py, ByteOwner { data: vec![42] }).unwrap();
            let view = PyMemoryView::from_owned_buffer(&owner, |o| &o.data).unwrap();
            let readonly: bool = view.getattr("readonly").unwrap().extract().unwrap();
            assert!(readonly);
        });
    }

    #[test]
    fn test_from_owned_buffer_empty() {
        Python::attach(|py| {
            let owner = Bound::new(py, ByteOwner { data: vec![] }).unwrap();
            let view = PyMemoryView::from_owned_buffer(&owner, |o| &o.data).unwrap();
            assert_eq!(view.len().unwrap(), 0);
        });
    }

    #[test]
    fn test_from_owned_buffer_static_data() {
        Python::attach(|py| {
            let owner = Bound::new(py, ByteOwner { data: vec![] }).unwrap();
            let view =
                PyMemoryView::from_owned_buffer(&owner, |_o| b"static data" as &[u8]).unwrap();
            let bytes: Vec<u8> = view.call_method0("tobytes").unwrap().extract().unwrap();
            assert_eq!(bytes, b"static data");
        });
    }
}
