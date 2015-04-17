use std;
use python::{PythonObject, Python, ToPythonPointer, PythonObjectDowncastError};
use objects::{PyObject, PyType, exc};
use objects::oldstyle::PyClass;
use ffi;
use libc;
use conversion::ToPyObject;
use std::ffi::CStr;

/// Represents a python exception that was raised.
#[derive(Clone, Debug)]
pub struct PyErr<'p> {
    /// Gets the type of the exception. This should be either a PyClass or a PyType.
    pub ptype : PyObject<'p>,
    /// Gets the value of the exception.
    /// This can be either an instance of ptype,
    /// a tuple of arguments to be passed to ptype's constructor,
    /// or a single argument to be passed to ptype's constructor.
    /// Call PyErr::instance() to get the exception instance in all cases.
    pub pvalue : Option<PyObject<'p>>,
    pub ptraceback : Option<PyObject<'p>> // is actually a PyTraceBack
}


/// Represents the result of a python call.
pub type PyResult<'p, T> = Result<T, PyErr<'p>>;

impl <'p> PyErr<'p> {
    /// Gets whether an error is present in the python interpreter's global state.
    #[inline]
    pub fn occurred(_ : Python<'p>) -> bool {
        unsafe { !ffi::PyErr_Occurred().is_null() }
    }

    /// Retrieves the current error from the python interpreter's global state.
    /// The error is cleared from the python interpreter.
    /// If no error is set, returns a SystemError.
    pub fn fetch(py : Python<'p>) -> PyErr<'p> {
        unsafe {
            let mut ptype      : *mut ffi::PyObject = std::mem::uninitialized();
            let mut pvalue     : *mut ffi::PyObject = std::mem::uninitialized();
            let mut ptraceback : *mut ffi::PyObject = std::mem::uninitialized();
            ffi::PyErr_Fetch(&mut ptype, &mut pvalue, &mut ptraceback);
            PyErr::new_from_ffi_tuple(py, ptype, pvalue, ptraceback)
        }
    }

    unsafe fn new_from_ffi_tuple(py: Python<'p>, ptype: *mut ffi::PyObject, pvalue: *mut ffi::PyObject, ptraceback: *mut ffi::PyObject) -> PyErr<'p> {
        // Note: must not panic to ensure all owned pointers get acquired correctly,
        // and because we mustn't panic in normalize().
        PyErr {
            ptype: if ptype.is_null() {
                        py.get_type::<exc::SystemError>().into_object()
                   } else {
                        PyObject::from_owned_ptr(py, ptype)
                   },
            pvalue: PyObject::from_owned_ptr_opt(py, pvalue),
            ptraceback: PyObject::from_owned_ptr_opt(py, ptraceback)
        }
    }
    
    /// Creates a new PyErr.
    /// If obj is a python exception instance, the PyErr will use that instance.
    /// If obj is a python exception type, the PyErr will (lazily) create a new instance of that type
    /// Otherwise, a TypeError is returned instead.
    pub fn new<O>(obj: O) -> PyErr<'p> where O: PythonObject<'p> {
        PyErr::new_from_object(obj.into_object())
    }
    
    fn new_from_object(obj: PyObject<'p>) -> PyErr<'p> {
        let py = obj.python();
        if unsafe { ffi::PyExceptionInstance_Check(obj.as_ptr()) } != 0 {
            PyErr {
                ptype: unsafe { PyObject::from_borrowed_ptr(py, ffi::PyExceptionInstance_Class(obj.as_ptr())) },
                pvalue: Some(obj),
                ptraceback: None
            }
        } else if unsafe { ffi::PyExceptionClass_Check(obj.as_ptr()) } != 0 {
            PyErr {
                ptype: obj,
                pvalue: None,
                ptraceback: None
            }
        } else {
            PyErr {
                ptype: py.get_type::<exc::TypeError>().into_object(),
                pvalue: "exceptions must derive from BaseException".to_py_object(py).ok(),
                ptraceback: None
            }
        }
    }

    /// Construct a new error, with the usual lazy initialization of python exceptions.
    /// `exc` is the exception type; usually one of the standard exceptions like `PyExc::runtime_error()`.
    /// `value` is the exception instance, or a tuple of arguments to pass to the exception constructor.
    #[inline]
    pub fn new_lazy_init(exc: PyType<'p>, value: Option<PyObject<'p>>) -> PyErr<'p> {
        PyErr {
            ptype: exc.into_object(),
            pvalue: value,
            ptraceback: None
        }
    }

    /// Print a standard traceback to sys.stderr.
    pub fn print(self) {
        self.restore();
        unsafe { ffi::PyErr_PrintEx(0) }
    }

    /// Print a standard traceback to sys.stderr.
    pub fn print_and_set_sys_last_vars(self) {
        self.restore();
        unsafe { ffi::PyErr_PrintEx(1) }
    }

    /// Return true if the current exception matches the exception in `exc`.
    /// If `exc` is a class object, this also returns `true` when `self` is an instance of a subclass.
    /// If `exc` is a tuple, all exceptions in the tuple (and recursively in subtuples) are searched for a match.
    #[inline]
    pub fn matches(&self, exc: &PyObject) -> bool {
        unsafe { ffi::PyErr_GivenExceptionMatches(self.ptype.as_ptr(), exc.as_ptr()) != 0 }
    }
    
    /// Normalizes the error. This ensures that the exception value is an instance of the exception type.
    pub fn normalize(&mut self) {
        // The normalization helper function involves temporarily moving out of the &mut self,
        // which requires some unsafe trickery:
        unsafe {
            std::ptr::write(self, std::ptr::read(self).into_normalized());
        }
        // This is safe as long as normalized() doesn't unwind due to a panic.
    }
    
    /// Helper function for normalizing the error by deconstructing and reconstructing the PyErr.
    /// Must not panic for safety in normalize()
    fn into_normalized(self) -> PyErr<'p> {
        let PyErr { ptype, pvalue, ptraceback } = self;
        let py = ptype.python();
        let mut ptype = ptype.steal_ptr();
        let mut pvalue = pvalue.steal_ptr();
        let mut ptraceback = ptraceback.steal_ptr();
        unsafe {
            ffi::PyErr_NormalizeException(&mut ptype, &mut pvalue, &mut ptraceback);
            PyErr::new_from_ffi_tuple(py, ptype, pvalue, ptraceback)
        }
    }
    
    /// Retrieves the exception type.
    /// If the exception type is an old-style class, returns oldstyle::PyClass.
    pub fn get_type(&self) -> PyType<'p> {
        let py = self.ptype.python();
        match self.ptype.clone().cast_into::<PyType>() {
            Ok(t)  => t,
            Err(_) =>
                match self.ptype.cast_as::<PyClass>() {
                    Ok(_)  => py.get_type::<PyClass>(),
                    Err(_) => py.None().get_type().clone()
                }
        }
    }

    /// Retrieves the exception instance for this error.
    /// This method takes &mut self because the error might need
    /// to be normalized in order to create the exception instance.
    pub fn instance(&mut self) -> PyObject<'p> {
        self.normalize();
        match self.pvalue {
            Some(ref instance) => instance.clone(),
            None => self.ptype.python().None()
        }
    }

    /// Restores the error by writing it to the python interpreter's global state.
    #[inline]
    pub fn restore(self) {
        let PyErr { ptype, pvalue, ptraceback } = self;
        unsafe {
            ffi::PyErr_Restore(ptype.steal_ptr(), pvalue.steal_ptr(), ptraceback.steal_ptr())
        }
    }
    
    /// Issue a warning message.
    /// May return a PyErr if warnings-as-errors is enabled.
    pub fn warn(py: Python<'p>, category: &PyObject, message: &CStr, stacklevel: i32) -> PyResult<'p, ()> {
        unsafe {
            error_on_minusone(py, ffi::PyErr_WarnEx(category.as_ptr(), message.as_ptr(), stacklevel as ffi::Py_ssize_t))
        }
    }
}

impl <'p> std::convert::From<PythonObjectDowncastError<'p>> for PyErr<'p> {
    fn from(err: PythonObjectDowncastError<'p>) -> PyErr<'p> {
        PyErr::new_lazy_init(err.0.get_type::<exc::TypeError>(), None)
    }
}

/// Construct PyObject from the result of a python FFI call that returns a new reference (owned pointer).
/// Returns Err(PyErr) if the pointer is null.
/// Unsafe because the pointer might be invalid.
#[inline]
pub unsafe fn result_from_owned_ptr(py : Python, p : *mut ffi::PyObject) -> PyResult<PyObject> {
    if p.is_null() {
        Err(PyErr::fetch(py))
    } else {
        Ok(PyObject::from_owned_ptr(py, p))
    }
}

/// Construct PyObject from the result of a python FFI call that returns a borrowed reference.
/// Returns Err(PyErr) if the pointer is null.
/// Unsafe because the pointer might be invalid.
#[inline]
pub unsafe fn result_from_borrowed_ptr(py : Python, p : *mut ffi::PyObject) -> PyResult<PyObject> {
    if p.is_null() {
        Err(PyErr::fetch(py))
    } else {
        Ok(PyObject::from_borrowed_ptr(py, p))
    }
}

pub unsafe fn result_cast_from_owned_ptr<'p, T>(py : Python<'p>, p : *mut ffi::PyObject) -> PyResult<'p, T>
    where T: ::python::PythonObjectWithCheckedDowncast<'p>
{
    if p.is_null() {
        Err(PyErr::fetch(py))
    } else {
        Ok(try!(PyObject::from_owned_ptr(py, p).cast_into()))
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

#[cfg(test)]
mod tests {
    use {Python, PyErr};
    use objects::{PyObject, exc};
    
    #[test]
    fn set_typeerror() {
        let gil = Python::acquire_gil();
        let py = gil.python();
        PyErr::new_lazy_init(py.get_type::<exc::TypeError>(), None).restore();
        assert!(PyErr::occurred(py));
        drop(PyErr::fetch(py));
    }
}


