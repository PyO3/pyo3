use crate::{ffi, instance::Bound, PyAny, PyErr, PyResult, Python};

use crate::create_exception;
use crate::exceptions::PyException;
use std::ffi::CString;
use std::fs::File;

create_exception!(crate, FileConversionError, PyException);

use crate::types::pyo3file::Pyo3File;

use crate::types::any::PyAnyMethods;

use std::mem;

#[cfg(unix)]
use std::os::fd::RawFd;
#[cfg(unix)]
use std::os::fd::{AsRawFd, FromRawFd};

#[cfg(windows)]
use std::os::windows::io::AsRawHandle;
#[cfg(windows)]
use std::os::windows::io::{FromRawHandle, RawHandle};
#[cfg(windows)]
use std::os::windows::prelude::IntoRawHandle;
#[cfg(windows)]
use winapi::um::handleapi::DuplicateHandle;
#[cfg(windows)]
use winapi::um::processthreadsapi::GetCurrentProcess;
#[cfg(windows)]
use winapi::um::winnt::{DUPLICATE_SAME_ACCESS, HANDLE};

/// Represents a Python `file` object.
///
/// Values of this type are accessed via PyO3's smart pointers, e.g. as
/// [`Py<PyFile>`][crate::Py] or [`Bound<'py, PyFile>`][Bound].
#[repr(transparent)]
pub struct PyFile(PyAny);

pyobject_native_type!(
    PyFile,
    ffi::PyFileObject,
    pyobject_native_static_type_object!(ffi::PyFile_Type),
    #checkfunction=ffi::PyFile_Check
);

impl PyFile {
    pub fn new(py: Python<'_>, pyo3_file: Pyo3File) -> PyResult<Bound<'_, PyAny>> {
        let file = pyo3_file.getfile();
        let fd = file.as_raw_fd();
        if fd < 0 {
            return Err(FileConversionError::new_err("Invalid file descriptor"));
        }

        let mode_cstr = CString::new(pyo3_file.mode.clone())
            .map_err(|_| FileConversionError::new_err("Invalid file mode"))?;

        let name_cstr = CString::new(pyo3_file.name.clone())
            .map_err(|_| FileConversionError::new_err("Invalid file name"))?;
        let encoding_cstr = CString::new(pyo3_file.encoding.clone())
            .map_err(|_| FileConversionError::new_err("Invalid encoding"))?;

        unsafe {
            let py_obj = ffi::PyFile_FromFd(
                fd,
                name_cstr.as_ptr(),
                mode_cstr.as_ptr(),
                -1,
                encoding_cstr.as_ptr(),
                std::ptr::null(),
                std::ptr::null(),
                1,
            );

            if py_obj.is_null() {
                Err(PyErr::fetch(py))
            } else {
                Ok(Bound::from_owned_ptr(py, py_obj as *mut _))
            }
        }
    }
}

impl<'py> crate::FromPyObject<'py> for Pyo3File {
    fn extract_bound(obj: &Bound<'py, PyAny>) -> PyResult<Self> {
        let fd: RawFd = unsafe { crate::ffi::PyObject_AsFileDescriptor(obj.as_ptr()) };
        if fd < 0 {
            return Err(PyErr::fetch(obj.py()));
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
        // Do not steal the handle from Python, as it is still used by the python object.
        mem::forget(file);

        let name: String = obj
            .getattr("name")?
            .extract()
            .unwrap_or_else(|_| "<unknown>".to_string());

        let mode: String = obj
            .getattr("mode")?
            .extract()
            .unwrap_or_else(|_| "r".to_string());

        let encoding: String = obj
            .getattr("encoding")?
            .extract()
            .unwrap_or_else(|_| "utf-8".to_string());

        Ok(Pyo3File::new(new_file, name, mode, encoding))
    }
}

impl<'py> crate::IntoPyObject<'py> for Pyo3File {
    type Target = PyAny;
    type Output = Bound<'py, PyAny>;
    type Error = PyErr;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        PyFile::new(py, self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::conversion::FromPyObject;
    use crate::types::pyo3file::Pyo3File;
    use crate::types::IntoPyDict;
    use std::ffi::CString;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_fake_python_file_with_invalid_fd() {
        Python::with_gil(|py| {
            let lambda_code = CString::new("lambda self: -1").unwrap();
            let name_code = CString::new("'fakefile'").unwrap();
            let mode_code = CString::new("'r'").unwrap();
            let encoding_code = CString::new("'utf-8'").unwrap();
            let class_code = CString::new("type('FakeFile', (), {})()").unwrap();

            let dict = [
                (
                    "fileno",
                    py.eval(lambda_code.as_c_str(), None, None).unwrap(),
                ),
                ("name", py.eval(name_code.as_c_str(), None, None).unwrap()),
                ("mode", py.eval(mode_code.as_c_str(), None, None).unwrap()),
                (
                    "encoding",
                    py.eval(encoding_code.as_c_str(), None, None).unwrap(),
                ),
            ]
            .into_py_dict(py)
            .unwrap();

            let py_file = py.eval(class_code.as_c_str(), Some(&dict), None).unwrap();

            let result = Pyo3File::extract_bound(&py_file);
            assert!(result.is_err(), "Expected error for invalid fd");
        });
    }

    #[test]
    fn test_pyo3file_to_python_preserves_attributes() {
        Python::with_gil(|py| {
            let mut temp_file = NamedTempFile::new().expect("Failed to create temp file");
            writeln!(temp_file, "Hello, world!").expect("Failed to write to temp file");
            let file = temp_file.reopen().expect("Failed to reopen temp file");

            let name = "myfile.txt".to_string();
            let mode = "r".to_string();
            let encoding = "utf-8".to_string();
            let pyo3_file = Pyo3File::new(file, name.clone(), mode.clone(), encoding.clone());

            let py_file_obj = PyFile::new(py, pyo3_file).expect("Failed to create PyFile");

            let py_mode: String = py_file_obj
                .getattr("mode")
                .expect("No 'mode' attr")
                .extract()
                .expect("Failed to extract mode");
            let py_encoding: String = py_file_obj
                .getattr("encoding")
                .expect("No 'encoding' attr")
                .extract()
                .expect("Failed to extract encoding");

            assert_eq!(py_mode, mode);
            assert_eq!(py_encoding, encoding);
        });
    }
}
