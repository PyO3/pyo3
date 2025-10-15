use crate::ffi_ptr_ext::FfiPtrExt;
use crate::py_result_ext::PyResultExt;
use crate::types::PyBytes;
use crate::{ffi, Bound, PyErr, PyResult, Python};
use pyo3_ffi::compat::{
    PyBytesWriter_Create, PyBytesWriter_Discard, PyBytesWriter_Finish, PyBytesWriter_GetData,
    PyBytesWriter_GetSize, PyBytesWriter_Grow, PyBytesWriter_WriteBytes,
    _PyBytesWriter_GetAllocated,
};
use std::io::IoSlice;
use std::ptr;
use std::ptr::NonNull;

pub struct PyBytesWriter<'py> {
    python: Python<'py>,
    writer: NonNull<ffi::PyBytesWriter>,
}

impl<'py> PyBytesWriter<'py> {
    #[inline]
    pub fn new(py: Python<'py>) -> PyResult<Self> {
        Self::with_capacity(py, 0)
    }

    #[inline]
    pub fn with_capacity(py: Python<'py>, capacity: usize) -> PyResult<Self> {
        match NonNull::new(unsafe { PyBytesWriter_Create(capacity as _) }) {
            Some(ptr) => Ok(PyBytesWriter {
                python: py,
                writer: ptr,
            }),
            None => Err(PyErr::fetch(py)),
        }
    }

    #[inline]
    pub fn capacity(&self) -> usize {
        unsafe { _PyBytesWriter_GetAllocated(self.writer.as_ptr()) as _ }
    }

    #[inline]
    pub fn len(&self) -> usize {
        unsafe { PyBytesWriter_GetSize(self.writer.as_ptr()) as _ }
    }

    #[inline]
    fn as_mut_ptr(&mut self) -> *mut u8 {
        unsafe { PyBytesWriter_GetData(self.writer.as_ptr()) as _ }
    }
}

impl<'py> TryInto<Bound<'py, PyBytes>> for PyBytesWriter<'py> {
    type Error = PyErr;

    #[inline]
    fn try_into(self) -> PyResult<Bound<'py, PyBytes>> {
        unsafe {
            PyBytesWriter_Finish(self.writer.as_ptr())
                .assume_owned_or_err(self.python)
                .cast_into_unchecked()
        }
    }
}

impl<'py> Drop for PyBytesWriter<'py> {
    #[inline]
    fn drop(&mut self) {
        unsafe { PyBytesWriter_Discard(self.writer.as_ptr()) }
    }
}

impl std::io::Write for PyBytesWriter<'_> {
    #[inline]
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let result = unsafe {
            PyBytesWriter_WriteBytes(self.writer.as_ptr(), buf.as_ptr() as _, buf.len() as _)
        };

        if result < 0 {
            Err(PyErr::fetch(self.python).into())
        } else {
            Ok(buf.len())
        }
    }

    #[inline]
    fn write_vectored(&mut self, bufs: &[IoSlice<'_>]) -> std::io::Result<usize> {
        let len = bufs.iter().map(|b| b.len()).sum();
        let mut pos = self.len();

        if unsafe { PyBytesWriter_Grow(self.writer.as_ptr(), len as _) } < 0 {
            return Err(PyErr::fetch(self.python).into());
        }

        for buf in bufs {
            // SAFETY: We have ensured enough capacity above.
            unsafe {
                ptr::copy_nonoverlapping(buf.as_ptr(), self.as_mut_ptr().add(pos), buf.len())
            };
            pos += buf.len();
        }
        Ok(len)
    }

    #[inline]
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }

    #[inline]
    fn write_all(&mut self, buf: &[u8]) -> std::io::Result<()> {
        self.write(buf)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn test_io_write() {
        Python::attach(|py| {
            let buf: [u8; _] = [1, 2, 3, 4];
            let mut writer = PyBytesWriter::new(py).unwrap();
            writer.write(&buf).unwrap();
            let bytes: Bound<'_, PyBytes> = writer.try_into().unwrap();
            assert_eq!(bytes, buf);
        })
    }

    #[test]
    fn test_io_write_vectored() {
        Python::attach(|py| {
            let bufs = [IoSlice::new(&[1, 2]), IoSlice::new(&[3, 4])];
            let mut writer = PyBytesWriter::new(py).unwrap();
            writer.write_vectored(&bufs).unwrap();
            let bytes: Bound<'_, PyBytes> = writer.try_into().unwrap();
            assert_eq!(bytes, [1, 2, 3, 4]);
        })
    }
}
