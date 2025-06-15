use crate::{ffi, instance::Bound, PyAny, PyErr, PyResult, Python, FromPyObject, IntoPyObject};
use nix::unistd::dup; // from nix crate
use std::fs::File;
use std::os::fd::{AsRawFd, FromRawFd};


/// Represents a Python `file` object.
///
/// Values of this type are accessed via PyO3's smart pointers, e.g. as
/// [`Py<PyFile>`][crate::Py] or [`Bound<'py, PyFile>`][Bound].
///
/// You can usually avoid directly working with this type
/// by using [`IntoPyObject`]
#[repr(transparent)]
pub struct PyFile(PyAny);

impl PyFile {
    /// Creates a new Python `file` object.
    pub fn new(py: Python<'_>, file: File) -> PyResult<Bound<'_, PyAny>> {
        let fd = file.as_raw_fd();

        // dup is used to avoid double close by creating a new file descriptor
        let dup_fd = dup(fd)
            .map_err(|e| PyErr::new::<crate::exceptions::PyOSError, _>(e.to_string()))?;

        unsafe {
            let py_obj = ffi::PyFile_FromFd(
                dup_fd,           // file descriptor
                std::ptr::null(), // name
                std::ptr::null(), // mode
                -1,               // buffering (default)
                std::ptr::null(), // encoding
                std::ptr::null(), // errors
                std::ptr::null(), // newline
                1,                // closefd (close when file is closed)
            );

            if py_obj.is_null() {
                Err(PyErr::fetch(py))
            } else {
                Ok(Bound::from_owned_ptr(py, py_obj as *mut _))
            }
        }
    }
}

impl<'py> FromPyObject<'py> for File {
    fn extract_bound(obj: &Bound<'py, PyAny>) -> PyResult<Self> {
        let mut _fd = -1;
        unsafe { _fd = ffi::PyObject_AsFileDescriptor(obj.as_ptr()); }

        if _fd < 0 {
            Err(PyErr::fetch(obj.py()))
        } else {
            let dup_fd = dup(_fd)
                .map_err(|e| PyErr::new::<crate::exceptions::PyOSError, _>(e.to_string()))?;
            unsafe { Ok(File::from_raw_fd(dup_fd)) }
        }
    }
}

impl<'py> IntoPyObject<'py> for File {
    type Target = PyAny;
    type Output = Bound<'py, PyAny>;
    type Error = PyErr;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        PyFile::new(py, self) 
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        conversion::IntoPyObject,
        types::{PyAnyMethods, PyFile},
        Python,
    };

    #[test]
    fn test_file() {

    }
}
