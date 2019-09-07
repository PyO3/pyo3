use pyo3::class::PyBufferProtocol;
use pyo3::exceptions::BufferError;
use pyo3::ffi;
use pyo3::prelude::*;
use pyo3::types::{IntoPyDict, PyAny};
use std::ffi::CStr;
use std::os::raw::{c_int, c_void};
use std::ptr;

#[pyclass]
struct TestClass {
    vec: Vec<u8>,
}

#[pyproto]
impl PyBufferProtocol for TestClass {
    fn bf_getbuffer(&self, view: *mut ffi::Py_buffer, flags: c_int) -> PyResult<()> {
        if view.is_null() {
            return Err(BufferError::py_err("View is null"));
        }

        unsafe {
            (*view).obj = ptr::null_mut();
        }

        if (flags & ffi::PyBUF_WRITABLE) == ffi::PyBUF_WRITABLE {
            return Err(BufferError::py_err("Object is not writable"));
        }

        let bytes = &self.vec;

        unsafe {
            (*view).buf = bytes.as_ptr() as *mut c_void;
            (*view).len = bytes.len() as isize;
            (*view).readonly = 1;
            (*view).itemsize = 1;

            (*view).format = ptr::null_mut();
            if (flags & ffi::PyBUF_FORMAT) == ffi::PyBUF_FORMAT {
                let msg = CStr::from_bytes_with_nul(b"B\0").unwrap();
                (*view).format = msg.as_ptr() as *mut _;
            }

            (*view).ndim = 1;
            (*view).shape = ptr::null_mut();
            if (flags & ffi::PyBUF_ND) == ffi::PyBUF_ND {
                (*view).shape = (&((*view).len)) as *const _ as *mut _;
            }

            (*view).strides = ptr::null_mut();
            if (flags & ffi::PyBUF_STRIDES) == ffi::PyBUF_STRIDES {
                (*view).strides = &((*view).itemsize) as *const _ as *mut _;
            }

            (*view).suboffsets = ptr::null_mut();
            (*view).internal = ptr::null_mut();
        }

        Ok(())
    }
}

#[test]
fn test_buffer() {
    let gil = Python::acquire_gil();
    let py = gil.python();

    let t = Py::new(
        py,
        TestClass {
            vec: vec![b' ', b'2', b'3'],
        },
    )
    .unwrap();

    let d = [("ob", t)].into_py_dict(py);
    py.run("assert bytes(ob) == b' 23'", None, Some(d)).unwrap();
}

fn byte_to_vec(py_byte_string: &PyAny) -> PyResult<Vec<u8>> {
    py_byte_string.extract()
}

fn byte_to_str(py_byte_string: &PyAny) -> PyResult<String> {
    py_byte_string.extract()
}

#[test]
fn test_byte_conversion() {
    let gil = Python::acquire_gil();
    let py = gil.python();
    let bytes = py.eval("b'I AM BYTE-STRING'", None, None).unwrap();
    let answer = "I AM BYTE-STRING";
    assert_eq!(byte_to_vec(bytes).unwrap(), answer.as_bytes().to_vec());
    assert_eq!(
        byte_to_str(bytes)
            .map_err(|err| {
                err.print_and_set_sys_last_vars(py);
            })
            .unwrap(),
        answer.to_string()
    );
}
