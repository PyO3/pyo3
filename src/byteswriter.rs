#[cfg(not(Py_LIMITED_API))]
use crate::{
    err::error_on_minusone,
    ffi::{
        self,
        compat::{
            PyBytesWriter_Create, PyBytesWriter_Discard, PyBytesWriter_Finish,
            PyBytesWriter_GetData, PyBytesWriter_GetSize, PyBytesWriter_Resize,
        },
    },
    ffi_ptr_ext::FfiPtrExt,
    py_result_ext::PyResultExt,
};
use crate::{types::PyBytes, Bound, IntoPyObject, PyErr, PyResult, Python};
use std::io::IoSlice;
#[cfg(not(Py_LIMITED_API))]
use std::{
    mem::ManuallyDrop,
    ptr::{self, NonNull},
};

pub struct PyBytesWriter<'py> {
    python: Python<'py>,
    #[cfg(not(Py_LIMITED_API))]
    writer: NonNull<ffi::compat::PyBytesWriter>,
    #[cfg(Py_LIMITED_API)]
    buffer: Vec<u8>,
}

impl<'py> PyBytesWriter<'py> {
    /// Create a new `PyBytesWriter` with a default initial capacity.
    #[inline]
    pub fn new(py: Python<'py>) -> PyResult<Self> {
        Self::with_capacity(py, 0)
    }

    /// Create a new `PyBytesWriter` with the specified initial capacity.
    #[inline]
    #[cfg_attr(Py_LIMITED_API, allow(clippy::unnecessary_wraps))]
    pub fn with_capacity(py: Python<'py>, capacity: usize) -> PyResult<Self> {
        #[cfg(not(Py_LIMITED_API))]
        {
            NonNull::new(unsafe { PyBytesWriter_Create(capacity as _) }).map_or_else(
                || Err(PyErr::fetch(py)),
                |writer| {
                    let mut writer = PyBytesWriter { python: py, writer };
                    if capacity > 0 {
                        // SAFETY: By setting the length to 0, we ensure no bytes are considered uninitialized.
                        unsafe { writer.set_len(0)? };
                    }
                    Ok(writer)
                },
            )
        }

        #[cfg(Py_LIMITED_API)]
        {
            Ok(PyBytesWriter {
                python: py,
                buffer: Vec::with_capacity(capacity),
            })
        }
    }

    /// Get the current length of the internal buffer.
    #[inline]
    pub fn len(&self) -> usize {
        #[cfg(not(Py_LIMITED_API))]
        unsafe {
            PyBytesWriter_GetSize(self.writer.as_ptr()) as _
        }

        #[cfg(Py_LIMITED_API)]
        {
            self.buffer.len()
        }
    }

    #[inline]
    #[cfg(not(Py_LIMITED_API))]
    fn as_mut_ptr(&mut self) -> *mut u8 {
        unsafe { PyBytesWriter_GetData(self.writer.as_ptr()) as _ }
    }

    /// Set the length of the internal buffer to `new_len`. The new bytes are uninitialized.
    ///
    /// # Safety
    /// The caller must ensure the new bytes are initialized. This will also make all pointers
    /// returned by `as_mut_ptr` invalid, so the caller must not hold any references to the buffer
    /// across this call.
    #[inline]
    #[cfg(not(Py_LIMITED_API))]
    unsafe fn set_len(&mut self, new_len: usize) -> PyResult<()> {
        unsafe {
            error_on_minusone(
                self.python,
                PyBytesWriter_Resize(self.writer.as_ptr(), new_len as _),
            )
        }
    }
}

impl<'py> TryFrom<PyBytesWriter<'py>> for Bound<'py, PyBytes> {
    type Error = PyErr;

    #[inline]
    fn try_from(value: PyBytesWriter<'py>) -> Result<Self, Self::Error> {
        let py = value.python;

        #[cfg(not(Py_LIMITED_API))]
        unsafe {
            PyBytesWriter_Finish(ManuallyDrop::new(value).writer.as_ptr())
                .assume_owned_or_err(py)
                .cast_into_unchecked()
        }

        #[cfg(Py_LIMITED_API)]
        {
            Ok(PyBytes::new(py, &value.buffer))
        }
    }
}

impl<'py> IntoPyObject<'py> for PyBytesWriter<'py> {
    type Target = PyBytes;
    type Output = Bound<'py, PyBytes>;
    type Error = PyErr;

    #[inline]
    fn into_pyobject(self, _py: Python<'py>) -> Result<Self::Output, Self::Error> {
        self.try_into()
    }
}

#[cfg(not(Py_LIMITED_API))]
impl<'py> Drop for PyBytesWriter<'py> {
    #[inline]
    fn drop(&mut self) {
        unsafe { PyBytesWriter_Discard(self.writer.as_ptr()) }
    }
}

#[cfg(not(Py_LIMITED_API))]
impl std::io::Write for PyBytesWriter<'_> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.write_all(buf)?;
        Ok(buf.len())
    }

    fn write_vectored(&mut self, bufs: &[IoSlice<'_>]) -> std::io::Result<usize> {
        let len = bufs.iter().map(|b| b.len()).sum();
        let pos = self.len();

        // SAFETY: We write the new uninitialized bytes below.
        unsafe { self.set_len(self.len() + len)? }

        // SAFETY: We ensured enough capacity above and the ptr will be valid because we will not be
        // resizing the buffer until we have written all the data.
        let mut ptr = unsafe { self.as_mut_ptr().add(pos) };

        for buf in bufs {
            // SAFETY: We have ensured enough capacity above.
            unsafe { ptr::copy_nonoverlapping(buf.as_ptr(), ptr, buf.len()) };

            // SAFETY: We just wrote buf.len() bytes
            ptr = unsafe { ptr.add(buf.len()) };
        }
        Ok(len)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }

    fn write_all(&mut self, buf: &[u8]) -> std::io::Result<()> {
        let len = buf.len();
        let pos = self.len();

        // SAFETY: We write the new uninitialized bytes below.
        unsafe { self.set_len(pos + len)? }

        // SAFETY: We have ensured enough capacity above.
        unsafe { ptr::copy_nonoverlapping(buf.as_ptr(), self.as_mut_ptr().add(pos), len) };

        Ok(())
    }
}

#[cfg(Py_LIMITED_API)]
impl std::io::Write for PyBytesWriter<'_> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.buffer.write(buf)
    }

    fn write_vectored(&mut self, bufs: &[IoSlice<'_>]) -> std::io::Result<usize> {
        self.buffer.write_vectored(bufs)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.buffer.flush()
    }

    fn write_all(&mut self, buf: &[u8]) -> std::io::Result<()> {
        self.buffer.write_all(buf)
    }

    fn write_fmt(&mut self, args: std::fmt::Arguments<'_>) -> std::io::Result<()> {
        self.buffer.write_fmt(args)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::PyBytesMethods;
    use std::io::Write;

    #[test]
    fn test_io_write() {
        Python::attach(|py| {
            let buf = b"hallo world";
            let mut writer = PyBytesWriter::new(py).unwrap();
            assert_eq!(writer.write(buf).unwrap(), 11);
            let bytes: Bound<'_, PyBytes> = writer.try_into().unwrap();
            assert_eq!(bytes.as_bytes(), buf);
        })
    }

    #[test]
    fn test_pre_allocated() {
        Python::attach(|py| {
            let buf = b"hallo world";
            let mut writer = PyBytesWriter::with_capacity(py, buf.len()).unwrap();
            assert_eq!(writer.len(), 0, "Writer position should be zero");
            assert_eq!(writer.write(buf).unwrap(), 11);
            let bytes: Bound<'_, PyBytes> = writer.try_into().unwrap();
            assert_eq!(bytes.as_bytes(), buf);
        })
    }

    #[test]
    fn test_io_write_vectored() {
        Python::attach(|py| {
            let bufs = [IoSlice::new(b"hallo "), IoSlice::new(b"world")];
            let mut writer = PyBytesWriter::new(py).unwrap();
            assert_eq!(writer.write_vectored(&bufs).unwrap(), 11);
            let bytes: Bound<'_, PyBytes> = writer.try_into().unwrap();
            assert_eq!(bytes.as_bytes(), b"hallo world");
        })
    }

    #[test]
    fn test_io_write_vectored_large() {
        Python::attach(|py| {
            let large_data = vec![b'\n'; 1024]; // 1 KB
            let bufs = [
                IoSlice::new(b"hallo"),
                IoSlice::new(&large_data),
                IoSlice::new(b"world"),
            ];
            let mut writer = PyBytesWriter::new(py).unwrap();
            assert_eq!(writer.write_vectored(&bufs).unwrap(), 1034);
            let bytes: Bound<'_, PyBytes> = writer.try_into().unwrap();
            assert!(bytes.as_bytes().starts_with(b"hallo\n"));
            assert!(bytes.as_bytes().ends_with(b"world"));
            assert_eq!(bytes.as_bytes().len(), 1034);
        })
    }

    #[test]
    fn test_large_data() {
        Python::attach(|py| {
            let mut writer = PyBytesWriter::new(py).unwrap();
            let large_data = vec![0; 1024]; // 1 KB
            writer.write_all(&large_data).unwrap();
            let bytes: Bound<'_, PyBytes> = writer.try_into().unwrap();
            assert_eq!(bytes.as_bytes(), large_data.as_slice());
        })
    }
}
