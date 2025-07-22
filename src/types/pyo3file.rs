use std::fs::File;
use crate::Bound;
use crate::PyAny;
use crate::PyResult;
use crate::PyErr;
use crate::Python;
use crate::types::any::PyAnyMethods;
use crate::ffi;
use std::mem;
use std::ffi::CString;

use crate::create_exception;
use crate::exceptions::PyException;

create_exception!(crate, FileConversionError, PyException);

#[cfg(unix)]
use std::os::fd::{FromRawFd, AsRawFd};

/// This is a structure to deal with python file
pub struct Pyo3File {
    /// Our rust file
    pub file: File,
    /// For python file, do nothing in rust
    ///
    /// See the Python `open` function:
    /// <https://docs.python.org/3/library/functions.html#open>
    pub mode: String,
    /// For python file, do nothing in rust
    ///
    /// See
    /// <https://doc.rust-lang.org/rustdoc/write-documentation/the-doc-attribute.html>
    pub encoding: String,
}

impl Pyo3File {
    ///
    fn new(file: File, mode: String, encoding: String) -> Self {
        Self {
            file,
            mode,
            encoding,
        }
    }

    ///
    pub fn getfile(&self) -> &File {
        &self.file
    }

    ///
    pub fn from_py_file(file_obj: &Bound<'_, PyAny>) -> PyResult<Self> {
        let fd = unsafe { crate::ffi::PyObject_AsFileDescriptor(file_obj.as_ptr()) };
        if fd < 0 {
            return Err(PyErr::fetch(file_obj.py()));
        }

        #[cfg(unix)]
        let file = unsafe { File::from_raw_fd(fd) };

        #[cfg(windows)]
        let file = unsafe {
            let raw_handle = libc::get_osfhandle(fd);
            if raw_handle == -1 {
                return Err(PyOSError::new_err(
                    "Cannot convert file descriptor to RawHandle",
                ));
            }
            File::from_raw_handle(raw_handle as _)
        };

        let new_file = file.try_clone()?;
        // Do not steal the handle from Python, as it is still used by the
        // python object.
        mem::forget(file);

        let mode: String = file_obj
            .getattr("mode")?
            .extract()
            .unwrap_or_else(|_| "r".to_string());

        let encoding: String = match mode.as_str() {
            m if m.contains('b') => String::from(""),
            _ => file_obj.getattr("encoding")?.extract()?,
        };

        Ok(Self::new(new_file, mode, encoding))
    }

    ///
    pub fn to_py_file<'py>(self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {

        let file = self.getfile();

        #[cfg(unix)]
        let fd = file.as_raw_fd();

        #[cfg(windows)]
        let fd = unsafe { 
            let handle = file.as_raw_handle();
            libc::open_osfhandle(handle as isize, 0) 
        };

        if fd < 0 {
            return Err(FileConversionError::new_err("Invalid file descriptor"));
        }

        let mode_cstr = CString::new(self.mode.clone())
            .map_err(|_| FileConversionError::new_err("Invalid file mode"))?;

        mem::forget(self.file);
        
        unsafe {
            let py_obj = match self.mode.as_str() {
                m if m.contains('b') => ffi::PyFile_FromFd(
                    fd,
                    std::ptr::null(),
                    mode_cstr.as_ptr(),
                    -1,
                    std::ptr::null(),
                    std::ptr::null(),
                    std::ptr::null(),
                    1,
                ),
                _ => {
                    let encoding_cstr = CString::new(self.encoding.clone()).map_err(|_| FileConversionError::new_err("Invalid encoding"))?;

                    ffi::PyFile_FromFd(
                    fd,
                    std::ptr::null(),
                    mode_cstr.as_ptr(),
                    -1,
                    encoding_cstr.as_ptr(),
                    std::ptr::null(),
                    std::ptr::null(),
                    1,
                    )
                }
            };


            if py_obj.is_null() {
                Err(PyErr::fetch(py))
            } else {
                Ok(Bound::from_owned_ptr(py, py_obj as *mut _))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_create_pyo3file() {
        let temp_file = NamedTempFile::new().expect("");
        let name: String = String::from("name");
        let mode: String = String::from("r");
        let encoding: String = String::from("utf-8");
        let pyo3_file: Pyo3File = Pyo3File::new(
            temp_file.into_file(),
            name.clone(),
            mode.clone(),
            encoding.clone(),
        );

        assert_eq!(pyo3_file.name, name.clone());
        assert_eq!(pyo3_file.mode, mode.clone());
        assert_eq!(pyo3_file.encoding, encoding.clone())
    }
}