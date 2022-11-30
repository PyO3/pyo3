use crate::buffer::{Element, PyBuffer};
use crate::err::PyResult;
use crate::types::PySequence;
use crate::{ffi, AsPyPointer, PyAny, Python};

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

    /// Creates a new Python memoryview from a raw pointer and length.
    ///
    /// Panics if out of memory.
    ///
    /// # Safety
    ///
    /// This function dereferences the raw pointer `ptr` as the
    /// leading pointer of a slice of length `len`. [As with
    /// `std::slice::from_raw_parts`, this is
    /// unsafe](https://doc.rust-lang.org/std/slice/fn.from_raw_parts.html#safety).
    pub unsafe fn from_ptr(
        py: Python<'_>,
        ptr: *const u8,
        len: usize,
        is_writable: bool,
    ) -> &PyMemoryView {
        let flags = if is_writable {
            ffi::PyBUF_WRITE
        } else {
            ffi::PyBUF_READ
        };

        py.from_owned_ptr(ffi::PyMemoryView_FromMemory(
            ptr as *mut _,
            len as ffi::Py_ssize_t,
            flags,
        ))
    }

    /// Returns the length of the memoryview.
    pub fn len(&self) -> usize {
        let size = unsafe { ffi::Py_SIZE(self.as_ptr()) };
        // non-negative Py_ssize_t should always fit into Rust usize
        size as usize
    }

    /// Checks if the memoryview is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns `self` cast as a `PySequence`.
    pub fn as_sequence(&self) -> &PySequence {
        unsafe { self.downcast_unchecked() }
    }

    /// Gets the Python string as a byte slice.
    #[inline]
    pub fn as_buffer<T: Element>(&self) -> PyResult<PyBuffer<T>> {
        PyBuffer::get(self)
    }
}

impl<'py> TryFrom<&'py PyAny> for &'py PyMemoryView {
    type Error = crate::PyErr;

    /// Creates a new Python `memoryview` object from another Python object that
    /// implements the buffer protocol.
    fn try_from(value: &'py PyAny) -> Result<Self, Self::Error> {
        PyMemoryView::from(value)
    }
}
