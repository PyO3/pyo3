use crate::{ffi, instance::Bound, PyAny, PyErr, PyResult, Python, FromPyObject, IntoPyObject};
use nix::unistd::dup; // from nix crate
use std::fs::File;
use std::os::fd::{AsRawFd, FromRawFd};
use std::ffi::CString;
use nix::fcntl::{fcntl, FcntlArg, OFlag};


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

    // We must respect the opening flags
    fn mode_string_from_flags(flags: OFlag) -> CString {
        let mode_str = match flags & OFlag::O_ACCMODE {
            OFlag::O_RDONLY => "r",
            OFlag::O_WRONLY => "w",
            OFlag::O_RDWR => "r+",
            _ => "r",
        };

        CString::new(mode_str).unwrap()
    }

    /// Creates a new Python `file` object.
    pub fn new(py: Python<'_>, file: File) -> PyResult<Bound<'_, PyAny>> {

        let fd = file.as_raw_fd();

        let flags_raw: i32 = fcntl(fd, FcntlArg::F_GETFL).expect("Error flag");
        let flags = OFlag::from_bits_truncate(flags_raw);
        let mode = PyFile::mode_string_from_flags(flags);

        let dup_fd = dup(fd)
            .map_err(|e| PyErr::new::<crate::exceptions::PyOSError, _>(e.to_string()))?;

        unsafe {
            let py_obj = ffi::PyFile_FromFd(
                dup_fd,           // file descriptor
                std::ptr::null(), // name
                mode.as_ptr(),    // mode
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
    use super::*;
    use std::fs::{File, OpenOptions};
    use std::io::{Write, Read, Seek, SeekFrom};
    use tempfile::NamedTempFile;
    use nix::fcntl::{fcntl, FcntlArg, OFlag};
    use crate::Python;
    use crate::types::any::PyAnyMethods;

    fn get_mode_flags(file: &File) -> nix::Result<OFlag> {
        let fd = file.as_raw_fd();
        let flags = fcntl(fd, FcntlArg::F_GETFL)?;
        Ok(OFlag::from_bits_truncate(flags))
    }

    // Convert file -> PyFile -> file to test the opening flags
    fn roundtrip_file(file: &File, content: String) {

        let orig_flags = get_mode_flags(file).expect("Failed to get original flags");
        Python::with_gil(|py| {
            // Convert Rust File to PyFile
            let pyfile = PyFile::new(py, file.try_clone().unwrap()).expect("Failed to create PyFile");

            // Convert back PyFile to Rust File
            let rust_file: File = pyfile.extract().expect("Failed to extract File");

            let roundtrip_flags = get_mode_flags(&rust_file).expect("Failed to get roundtrip flags");

            assert_eq!(
                orig_flags,
                roundtrip_flags,
                "Access mode flags should match"
            );

            // If readable, verify content
            if (orig_flags & OFlag::O_ACCMODE) != OFlag::O_WRONLY {
                let mut buf = String::new();
                rust_file.try_clone().unwrap().take(100).read_to_string(&mut buf).unwrap();
                assert!(buf.contains(&content));
            }
        });
    }

    #[test]
    fn test_read_only_mode() {
        let mut tmp = NamedTempFile::new().unwrap();
        let content: String = String::from("hello read-only");
        writeln!(tmp, "{}", content).unwrap();
        tmp.as_file_mut().seek(SeekFrom::Start(0)).unwrap();

        let file = OpenOptions::new().read(true).open(tmp.path()).unwrap();
        roundtrip_file(&file, content);
    }

    #[test]
    fn test_write_only_mode() {
        let tmp = NamedTempFile::new().unwrap();

        let file = OpenOptions::new().write(true).open(tmp.path()).unwrap();
        roundtrip_file(&file, String::new());
    }

    #[test]
    fn test_read_write_mode() {
        let mut tmp = NamedTempFile::new().unwrap();
        let content: String = String::from("hello read-write");
        writeln!(tmp, "{}", content).unwrap();
        tmp.as_file_mut().seek(SeekFrom::Start(0)).unwrap();

        let file = OpenOptions::new().read(true).write(true).open(tmp.path()).unwrap();
        roundtrip_file(&file, content);
    }
}
