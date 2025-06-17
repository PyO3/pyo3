use crate::{ffi, instance::Bound, PyAny, PyErr, PyResult, Python};

use std::fs::File;
#[cfg(unix)]
use std::os::fd::{AsRawFd, FromRawFd};
use std::ffi::CString;
use crate::create_exception;
use crate::exceptions::PyException;

create_exception!(crate, FileConversionError, PyException);

use crate::types::pyo3file::Pyo3File;

use crate::exceptions::PyOSError;
#[cfg(unix)]
use std::os::fd::RawFd;

use crate::types::any::PyAnyMethods;

#[cfg(unix)]
use nix::unistd::dup;

#[cfg(windows)]
use std::os::windows::io::AsRawHandle;
#[cfg(windows)]
use winapi::um::handleapi::DuplicateHandle;
#[cfg(windows)]
use winapi::um::processthreadsapi::GetCurrentProcess;
#[cfg(windows)]
use winapi::um::winnt::{DUPLICATE_SAME_ACCESS, HANDLE};
#[cfg(windows)]
use std::os::windows::prelude::IntoRawHandle;
#[cfg(windows)]
use std::os::windows::io::{FromRawHandle, RawHandle};

/// Represents a Python `file` object.
///
/// Values of this type are accessed via PyO3's smart pointers, e.g. as
/// [`Py<PyFile>`][crate::Py] or [`Bound<'py, PyFile>`][Bound].
///
/// You can usually avoid directly working with this type
/// by using [`IntoPyObject`]
#[repr(transparent)]
pub struct PyFile(PyAny);

pyobject_native_type!(
    PyFile,
    ffi::PyFileObject,
    pyobject_native_static_type_object!(ffi::PyFile_Type),
    #checkfunction=ffi::PyFile_Check
);

impl PyFile {

    #[cfg(unix)]
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

        let dup_fd = dup(fd).map_err(|e| PyErr::new::<PyOSError, _>(e.to_string()))?;

        unsafe {
            let py_obj = ffi::PyFile_FromFd(
                dup_fd,
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

    #[cfg(windows)]
    pub fn new(py: Python<'_>, pyo3_file: Pyo3File) -> PyResult<Bound<'_, PyAny>> {
        let handle: HANDLE = pyo3_file.file.as_raw_handle() as HANDLE;
        let mut dup_handle: HANDLE = ptr::null_mut();

        let success = unsafe {
            DuplicateHandle(
                GetCurrentProcess(),
                handle,
                GetCurrentProcess(),
                &mut dup_handle,
                0,
                1,
                DUPLICATE_SAME_ACCESS,
            )
        };

        if success == 0 {
            return Err(FileConversionError::new_err("Failed to duplicate handle"));
        }

        // Convert HANDLE to RawFd
        let fd = unsafe { libc::open_osfhandle(dup_handle as isize, 0) };
        if fd < 0 {
            return Err(FileConversionError::new_err("Failed to convert handle to fd"));
        }

        let mode_cstr = CString::new(pyo3_file.mode.clone())
            .map_err(|_| FileConversionError::new_err("Invalid mode string"))?;
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

    #[cfg(unix)]
    fn extract_bound(obj: &Bound<'py, PyAny>) -> PyResult<Self> {
        let fd: RawFd = unsafe { crate::ffi::PyObject_AsFileDescriptor(obj.as_ptr()) };
        if fd < 0 {
            return Err(PyErr::fetch(obj.py()));
        }

        let dup_fd = dup(fd).map_err(|e| PyErr::new::<PyOSError, _>(e.to_string()))?;
        let file = unsafe { File::from_raw_fd(dup_fd) };

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

        Ok(Pyo3File::new(
            file,
            name.clone(),
            name.clone(),
            mode,
            encoding,
        ))
    }

    #[cfg(windows)]
    fn extract_bound(obj: &Bound<'py, PyAny>) -> PyResult<Self> {
        let fd: i32 = unsafe { crate::ffi::PyObject_AsFileDescriptor(obj.as_ptr()) };
        if fd < 0 {
            return Err(PyErr::fetch(obj.py()));
        }

        // Convert fd to raw HANDLE
        let raw_handle = unsafe { libc::get_osfhandle(fd) };
        if raw_handle == -1 {
            return Err(FileConversionError::new_err("Invalid handle from fd"));
        }

        let mut dup_handle: HANDLE = ptr::null_mut();
        let success = unsafe {
            DuplicateHandle(
                GetCurrentProcess(),
                raw_handle as HANDLE,
                GetCurrentProcess(),
                &mut dup_handle,
                0,
                1,
                DUPLICATE_SAME_ACCESS,
            )
        };

        if success == 0 {
            return Err(FileConversionError::new_err("Failed to duplicate handle"));
        }

        let file = unsafe { File::from_raw_handle(dup_handle as RawHandle) };

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

        Ok(Pyo3File::new(
            file,
            name.clone(),
            name.clone(),
            mode,
            encoding,
        ))
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
