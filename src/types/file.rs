use crate::{ffi, instance::Bound, PyAny, PyErr, PyResult, Python};

use std::fs::File;
use std::ffi::CString;
use crate::create_exception;
use crate::exceptions::PyException;

create_exception!(crate, FileConversionError, PyException);

use crate::types::pyo3file::Pyo3File;

use crate::exceptions::PyOSError;

use crate::types::any::PyAnyMethods;

#[cfg(unix)]
use std::os::fd::{AsRawFd, FromRawFd};
#[cfg(unix)]
use std::os::fd::RawFd;
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
            name,
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
            name,
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{IntoPyObject, Python};
    use tempfile::NamedTempFile;
    use pyo3_ffi::c_str;
    use crate::prelude::PyModule;
    use crate::Py;

    fn create_file_like_object_with_fd<'py>(
        py: Python<'py>,
        fd: i32,
        name: String,
        mode: String,
        encoding: String
    ) -> &'py PyAny {

        let code = format!(
        "
class FileLike:
    def __init__(self):
        self.name = {name:?}
        self.mode = {mode:?}
        self.encoding = {encoding:?}
    def fileno(self):
        return {fd}
",
        name = name,
        mode = mode,
        encoding = encoding,
        fd = fd
    );

        Python::with_gil(|py| {
            let obj: Py<PyAny> = PyModule::from_code(
                py,
                pyo3_ffi::c_str!(&code),
                c_str!(""),
                c_str!(""),
            )?
                .getattr("FileLike")?
                .into();

            Ok(obj)})

    }

    #[test]
    fn test_create_object_from_pyo3file() {
        Python::with_gil(|py| {
        let temp_file = NamedTempFile::new().expect("");
        let name: String = String::from("name");
        let mode: String = String::from("r");
        let encoding: String = String::from("utf-8");
        let pyo3_file: Pyo3File = Pyo3File::new(
            temp_file.into_file(),
            name.clone(),
            mode.clone(),
            encoding.clone());

        let pyfile = pyo3_file.into_pyobject(py).expect("");

        let name_pyfile: String = pyfile
            .getattr("name").expect("")
            .extract()
            .unwrap_or_else(|_| "<unknown>".to_string());

        let mode_pyfile: String = pyfile
            .getattr("mode").expect("")
            .extract()
            .unwrap_or_else(|_| "r".to_string());

        let encoding_pyfile: String = pyfile
            .getattr("encoding").expect("")
            .extract()
            .unwrap_or_else(|_| "utf-8".to_string());

        //assert_eq!(name_pyfile.clone(), name.clone());
        assert_eq!(mode_pyfile.clone(), mode.clone());
        assert_eq!(encoding_pyfile.clone(), encoding.clone())
        })
    }

    #[test]
    fn test_create_pyo3file_from_object_() {
        Python::with_gil(|py| {
            let name: String = String::from("name");
            let mode: String = String::from("r");
            let encoding: String = String::from("utf-8");
            let temp_file = NamedTempFile::new().expect("");
            let file: File = temp_file.into_file();
            let fd: i32 = file.as_raw_fd();

            let pyfile: PyAny = create_file_like_object(py, name, mode, encoding, fd);

            let name_pyfile: String = pyfile
                .getattr("name").expect("")
                .extract()
                .unwrap_or_else(|_| "<unknown>".to_string());

            let mode_pyfile: String = pyfile
                .getattr("mode").expect("")
                .extract()
                .unwrap_or_else(|_| "r".to_string());

            let encoding_pyfile: String = pyfile
                .getattr("encoding").expect("")
                .extract()
                .unwrap_or_else(|_| "utf-8".to_string());


        })
    }
}
