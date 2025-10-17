use crate::types::PyBytes;
#[cfg(not(Py_LIMITED_API))]
use crate::{
    ffi::{
        self,
        compat::{
            PyBytesWriter_Create, PyBytesWriter_Discard, PyBytesWriter_Finish,
            PyBytesWriter_GetData, PyBytesWriter_GetSize, PyBytesWriter_Grow,
            PyBytesWriter_WriteBytes, _PyBytesWriter_GetAllocated,
        },
    },
    ffi_ptr_ext::FfiPtrExt,
    py_result_ext::PyResultExt,
};
use crate::{Bound, IntoPyObject, PyErr, PyResult, Python};
use std::io::IoSlice;
#[cfg(not(Py_LIMITED_API))]
use std::{
    mem::ManuallyDrop,
    ptr::{self, NonNull},
};

pub struct PyBytesWriter<'py> {
    python: Python<'py>,
    #[cfg(not(Py_LIMITED_API))]
    writer: NonNull<ffi::PyBytesWriter>,
    #[cfg(Py_LIMITED_API)]
    buffer: Vec<u8>,
}

impl<'py> PyBytesWriter<'py> {
    #[inline]
    pub fn new(py: Python<'py>) -> PyResult<Self> {
        Self::with_capacity(py, 0)
    }

    #[inline]
    pub fn with_capacity(py: Python<'py>, capacity: usize) -> PyResult<Self> {
        #[cfg(not(Py_LIMITED_API))]
        {
            NonNull::new(unsafe { PyBytesWriter_Create(capacity as _) })
                .map(|writer| PyBytesWriter { python: py, writer })
                .ok_or_else(|| PyErr::fetch(py))
        }

        #[cfg(Py_LIMITED_API)]
        {
            Ok(PyBytesWriter {
                python: py,
                buffer: Vec::with_capacity(capacity),
            })
        }
    }

    #[inline]
    pub fn capacity(&self) -> usize {
        #[cfg(not(Py_LIMITED_API))]
        unsafe {
            _PyBytesWriter_GetAllocated(self.writer.as_ptr()) as _
        }

        #[cfg(Py_LIMITED_API)]
        {
            self.buffer.capacity()
        }
    }

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
}

#[cfg(not(Py_LIMITED_API))]
impl<'py> TryFrom<PyBytesWriter<'py>> for Bound<'py, PyBytes> {
    type Error = PyErr;

    #[inline]
    fn try_from(value: PyBytesWriter<'py>) -> Result<Self, Self::Error> {
        let py = value.python;
        unsafe {
            PyBytesWriter_Finish(ManuallyDrop::new(value).writer.as_ptr())
                .assume_owned_or_err(py)
                .cast_into_unchecked()
        }
    }
}

#[cfg(Py_LIMITED_API)]
impl<'py> From<PyBytesWriter<'py>> for Bound<'py, PyBytes> {
    #[inline]
    fn from(writer: PyBytesWriter<'py>) -> Self {
        PyBytes::new(writer.python, &writer.buffer)
    }
}

impl<'py> IntoPyObject<'py> for PyBytesWriter<'py> {
    type Target = PyBytes;
    type Output = Bound<'py, PyBytes>;
    type Error = PyErr;

    #[inline]
    fn into_pyobject(self, _py: Python<'py>) -> Result<Self::Output, Self::Error> {
        self.try_into().map_err(Into::into)
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

#[cfg(Py_LIMITED_API)]
impl std::io::Write for PyBytesWriter<'_> {
    #[inline]
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.buffer.write(buf)
    }

    #[inline]
    fn write_vectored(&mut self, bufs: &[IoSlice<'_>]) -> std::io::Result<usize> {
        self.buffer.write_vectored(bufs)
    }

    #[inline]
    fn flush(&mut self) -> std::io::Result<()> {
        self.buffer.flush()
    }

    #[inline]
    fn write_all(&mut self, buf: &[u8]) -> std::io::Result<()> {
        self.buffer.write_all(buf)
    }

    #[inline]
    fn write_fmt(&mut self, args: std::fmt::Arguments<'_>) -> std::io::Result<()> {
        self.buffer.write_fmt(args)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn test_io_write() {
        Python::attach(|py| {
            let buf = [1, 2, 3, 4];
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
