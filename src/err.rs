use std;
use {PyObject, PythonObject, PyType, Python};
use pyptr::{PyPtr, PythonPointer};
use ffi;
use libc;
use conversion::ToPyObject;

/// Represents a python exception that was raised.
#[derive(Clone, Show)]
pub struct PyErr<'p> {
    /// Gets the type of the exception. This should be either a PyClass or a PyType.
    pub ptype : PyPtr<'p, PyObject<'p>>,
    /// Gets the value of the exception.
    /// This can be either an instance of ptype,
    /// a tuple of arguments to be passed to ptype's constructor,
    /// or a single argument to be passed to ptype's constructor.
    /// Call PyErr::instance() to get the exception instance in all cases.
    pub pvalue : Option<PyPtr<'p, PyObject<'p>>>,
    pub ptraceback : Option<PyPtr<'p, PyObject<'p>>> // is actually a PyTraceBack
}


/// Represents the result of a python call.
pub type PyResult<'p, T> = Result<T, PyErr<'p>>;
pub type PyPtrResult<'p, T> = PyResult<'p, PyPtr<'p, T>>;

impl <'p> PyErr<'p> {
    /// Gets whether an error is present in the python interpreter's global state.
    #[inline]
    pub fn occurred(_ : Python<'p>) -> bool {
        unsafe { !ffi::PyErr_Occurred().is_null() }
    }

    /// Retrieves the current error from the python interpreter's global state.
    /// The error is cleared from the python interpreter.
    /// If no error is set, returns a RuntimeError.
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
                        unimplemented!()
                   } else {
                        PyPtr::from_owned_ptr(py, ptype)
                   },
            pvalue: PyPtr::from_owned_ptr_opt(py, pvalue),
            ptraceback: PyPtr::from_owned_ptr_opt(py, ptraceback)
        }
    }

    /// Construct a new error.
    /// `exc` is the exception type; usually one of the standard exceptions like `PyExc::runtime_error()`.
    /// `value` is the exception instance, or a tuple of arguments to pass to the exception constructor
    pub fn new(exc: &PyObject<'p>, value: Option<PyPtr<'p, PyObject<'p>>>) -> PyErr<'p> {
        PyErr {
            ptype: PyPtr::new(exc),
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

    /// Print a warning message to sys.stderr when an exception has been set but it is impossible for the interpreter to actually raise the exception.
    /// It is used, for example, when an exception occurs in an __del__() method..
    pub fn write_unraisable(self, context: &PyObject<'p>) {
        self.restore();
        unsafe { ffi::PyErr_WriteUnraisable(context.as_ptr()) }
    }

    /// Return true if the current exception matches the exception in `exc`.
    /// If `exc` is a class object, this also returns `true` when `self` is an instance of a subclass.
    /// If `exc` is a tuple, all exceptions in the tuple (and recursively in subtuples) are searched for a match.
    pub fn matches(&self, exc: &PyObject) -> bool {
        unsafe { ffi::PyErr_GivenExceptionMatches(self.ptype.as_ptr(), exc.as_ptr()) != 0 }
    }
    
    /// Normalizes the error. This ensures that the exception value is an instance of the exception type.
    pub fn normalize(&mut self) {
        // The normalization helper function involves temporarily moving out of the &mut self,
        // which requires some unsafe trickery:
        unsafe {
            std::ptr::write(self, std::ptr::read(self).normalized());
        }
        // This is safe as long as normalized() doesn't unwind due to a panic.
    }
    
    /// Helper function for normalizing the error by deconstructing and reconstructing the PyErr.
    /// Must not panic for safety in normalize()
    fn normalized(self) -> PyErr<'p> {
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
    
    /// Retrieves the exception instance for this error.
    /// This method takes &mut self because the error might need to be normalized in order to create the exception instance.
    pub fn instance(&mut self) -> &PyObject<'p> {
        self.normalize();
        match self.pvalue {
            Some(ref instance) => &**instance,
            None => self.ptype.python().None()
        }
    }

    /// Restores the error by writing it to the python interpreter's global state.
    pub fn restore(self) {
        let PyErr { ptype, pvalue, ptraceback } = self;
        unsafe {
            ffi::PyErr_Restore(ptype.steal_ptr(), pvalue.steal_ptr(), ptraceback.steal_ptr())
        }
    }
}

/// Contains getter functions for the python exception types.
#[allow(non_snake_case)]
pub mod exception_types {
    
    macro_rules! exc_getter(
        ($name:ident) => (
            #[inline]
            pub fn $name(py: ::python::Python) -> &::object::PyObject {
                unsafe { ::object::PyObject::from_ptr(py, ::ffi::$name) }
            }
        )
    );
    
    exc_getter!(PyExc_BaseException);
    exc_getter!(PyExc_Exception);
    exc_getter!(PyExc_StandardError);
    exc_getter!(PyExc_LookupError);
    exc_getter!(PyExc_AssertionError);
    exc_getter!(PyExc_AttributeError);
    exc_getter!(PyExc_EOFError);
    exc_getter!(PyExc_EnvironmentError);
    exc_getter!(PyExc_FloatingPointError);
    exc_getter!(PyExc_IOError);
    exc_getter!(PyExc_ImportError);
    exc_getter!(PyExc_IndexError);
    exc_getter!(PyExc_KeyError);
    exc_getter!(PyExc_KeyboardInterrupt);
    exc_getter!(PyExc_MemoryError);
    exc_getter!(PyExc_NameError);
    exc_getter!(PyExc_NotImplementedError);
    exc_getter!(PyExc_OSError);
    exc_getter!(PyExc_OverflowError);
    exc_getter!(PyExc_ReferenceError);
    exc_getter!(PyExc_RuntimeError);
    exc_getter!(PyExc_SyntaxError);
    exc_getter!(PyExc_SystemError);
    exc_getter!(PyExc_SystemExit);
    exc_getter!(PyExc_TypeError);
    exc_getter!(PyExc_ValueError);
    #[cfg(target_os="windows")]
    exc_getter!(PyExc_WindowsError);
    exc_getter!(PyExc_ZeroDivisionError);
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

#[cfg(test)]
mod tests {
    use {Python, PyType, PyErr};
    
    #[test]
    fn set_typeerror() {
        let gil = Python::acquire_gil();
        let py = gil.python();
        PyErr::new(::err::exception_types::PyExc_TypeError(py), None).restore();
        assert!(PyErr::occurred(py));
        drop(PyErr::fetch(py))
    }
}


