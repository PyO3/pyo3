use std;
use {PyObject, PythonObject, PyType, Python, PyPtr};
use pyptr::PythonPointer;
use ffi;
use libc;

/// Represents a python exception that was raised.
#[derive(Clone, Show)]
pub struct PyErr<'p> {
    ptype : Option<PyPtr<'p, PyObject<'p>>>,
    pvalue : Option<PyPtr<'p, PyObject<'p>>>,
    ptraceback : Option<PyPtr<'p, PyObject<'p>>>
}


/// Represents the result of a python call.
pub type PyResult<'p, T> = Result<T, PyErr<'p>>;
pub type PyPtrResult<'p, T> = PyResult<'p, PyPtr<'p, T>>;

impl <'p> PyErr<'p> {
    /// Gets whether an error is present in the python interpreter's global state.
    pub fn occurred(_ : Python<'p>) -> bool {
        unsafe { !ffi::PyErr_Occurred().is_null() }
    }

    /// Retrieves the current error from the python interpreter's global state.
    /// The error is cleared from the python interpreter.
    pub fn fetch(py : Python<'p>) -> PyErr<'p> {
        unsafe {
            let mut ptype      : *mut ffi::PyObject = std::mem::uninitialized();
            let mut pvalue     : *mut ffi::PyObject = std::mem::uninitialized();
            let mut ptraceback : *mut ffi::PyObject = std::mem::uninitialized();
            ffi::PyErr_Fetch(&mut ptype, &mut pvalue, &mut ptraceback);
            PyErr {
                ptype: PyPtr::from_owned_ptr_opt(py, ptype),
                pvalue: PyPtr::from_owned_ptr_opt(py, pvalue),
                ptraceback: PyPtr::from_owned_ptr_opt(py, pvalue)
            }
        }
    }

    /// Restores the error by writing it to the python interpreter's global state.
    pub fn restore(self) {
        let PyErr { ptype, pvalue, ptraceback } = self;
        unsafe {
            ffi::PyErr_Restore(ptype.steal_ptr(), pvalue.steal_ptr(), ptraceback.steal_ptr())
        }
    }

    #[allow(unused_variables)]
    pub fn type_error(obj : &PyObject<'p>, expected_type : &PyType<'p>) -> PyErr<'p> {
        let py = obj.python();
        PyErr {
            ptype: Some(unsafe { PyPtr::from_borrowed_ptr(py, ffi::PyExc_TypeError) }),
            pvalue: None,
            ptraceback: None
        }
    }
}

/// Construct PyObject from the result of a python FFI call that returns a new reference (owned pointer).
/// Returns Err(PyErr) if the pointer is null.
/// Unsafe because the pointer might be invalid.
#[inline]
pub unsafe fn result_from_owned_ptr(py : Python, p : *mut ffi::PyObject) -> PyResult<PyPtr<PyObject>> {
    if p.is_null() {
        Err(PyErr::fetch(py))
    } else {
        Ok(PyPtr::from_owned_ptr(py, p))
    }
}

/// Returns Ok if the error code is 0.
#[inline]
pub fn error_on_nonzero(py : Python, result : libc::c_int) -> PyResult<()> {
    if result == 0 {
        Ok(())
    } else {
        Err(PyErr::fetch(py))
    }
}

/// Returns Ok if the error code is not -1.
#[inline]
pub fn error_on_minusone(py : Python, result : libc::c_int) -> PyResult<()> {
    if result != -1 {
        Ok(())
    } else {
        Err(PyErr::fetch(py))
    }
}



