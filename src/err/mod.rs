// Copyright (c) 2017-present PyO3 Project and Contributors

use crate::panic::PanicException;
use crate::type_object::PyTypeObject;
use crate::types::PyType;
use crate::{
    exceptions::{self, PyBaseException},
    ffi,
};
use crate::{
    AsPyPointer, FromPyPointer, IntoPy, Py, PyAny, PyNativeType, PyObject, Python,
    ToBorrowedObject, ToPyObject,
};
use std::borrow::Cow;
use std::cell::UnsafeCell;
use std::ffi::CString;
use std::os::raw::c_char;
use std::os::raw::c_int;
use std::ptr::NonNull;

mod err_state;
mod impls;

pub use err_state::PyErrArguments;
use err_state::{boxed_args, PyErrState, PyErrStateNormalized};

/// Represents a Python exception that was raised.
pub struct PyErr {
    // Safety: can only hand out references when in the "normalized" state. Will never change
    // after normalization.
    //
    // The state is temporarily removed from the PyErr during normalization, to avoid
    // concurrent modifications.
    state: UnsafeCell<Option<PyErrState>>,
}

unsafe impl Send for PyErr {}
unsafe impl Sync for PyErr {}

/// Represents the result of a Python call.
pub type PyResult<T> = Result<T, PyErr>;

/// Error that indicates a failure to convert a PyAny to a more specific Python type.
#[derive(Debug)]
pub struct PyDowncastError<'a> {
    from: &'a PyAny,
    to: Cow<'static, str>,
}

impl<'a> PyDowncastError<'a> {
    pub fn new(from: &'a PyAny, to: impl Into<Cow<'static, str>>) -> Self {
        PyDowncastError {
            from,
            to: to.into(),
        }
    }
}

impl PyErr {
    /// Creates a new PyErr of type `T`.
    ///
    /// `value` can be:
    /// * a tuple: the exception instance will be created using Python `T(*tuple)`
    /// * any other value: the exception instance will be created using Python `T(value)`
    ///
    /// Note: if `value` is not `Send` or `Sync`, consider using `PyErr::from_instance` instead.
    ///
    /// Panics if `T` is not a Python class derived from `BaseException`.
    ///
    /// Example:
    /// ```ignore
    /// return Err(PyErr::new::<exceptions::PyTypeError, _>("Error message"));
    /// ```
    ///
    /// In most cases, you can use a concrete exception's constructor instead, which is equivalent:
    /// ```ignore
    /// return Err(exceptions::PyTypeError::new_err("Error message"));
    /// ```
    pub fn new<T, A>(args: A) -> PyErr
    where
        T: PyTypeObject,
        A: PyErrArguments + Send + Sync + 'static,
    {
        Python::with_gil(|py| PyErr::from_type(T::type_object(py), args))
    }

    /// Constructs a new error, with the usual lazy initialization of Python exceptions.
    ///
    /// `exc` is the exception type; usually one of the standard exceptions
    /// like `exceptions::PyRuntimeError`.
    /// `args` is the a tuple of arguments to pass to the exception constructor.
    pub fn from_type<A>(ty: &PyType, args: A) -> PyErr
    where
        A: PyErrArguments + Send + Sync + 'static,
    {
        if unsafe { ffi::PyExceptionClass_Check(ty.as_ptr()) } == 0 {
            return exceptions_must_derive_from_base_exception(ty.py());
        }

        PyErr::from_state(PyErrState::Lazy {
            ptype: ty.into(),
            pvalue: boxed_args(args),
        })
    }

    /// Creates a new PyErr.
    ///
    /// `obj` must be an Python exception instance, the PyErr will use that instance.
    /// If `obj` is a Python exception type object, the PyErr will (lazily) create a new
    /// instance of that type.
    /// Otherwise, a `TypeError` is created instead.
    ///
    /// # Examples
    /// ```rust
    /// use pyo3::{Python, PyErr, IntoPy, exceptions::PyTypeError, types::PyType};
    /// Python::with_gil(|py| {
    ///     // Case #1: Exception instance
    ///     let err = PyErr::from_instance(PyTypeError::new_err("some type error",).instance(py));
    ///     assert_eq!(err.to_string(), "TypeError: some type error");
    ///
    ///     // Case #2: Exception type
    ///     let err = PyErr::from_instance(PyType::new::<PyTypeError>(py));
    ///     assert_eq!(err.to_string(), "TypeError: ");
    ///
    ///     // Case #3: Invalid exception value
    ///     let err = PyErr::from_instance("foo".into_py(py).as_ref(py));
    ///     assert_eq!(err.to_string(), "TypeError: exceptions must derive from BaseException");
    /// });
    /// ```
    pub fn from_instance(obj: &PyAny) -> PyErr {
        let ptr = obj.as_ptr();

        let state = if unsafe { ffi::PyExceptionInstance_Check(ptr) } != 0 {
            PyErrState::Normalized(PyErrStateNormalized {
                ptype: unsafe {
                    Py::from_borrowed_ptr(obj.py(), ffi::PyExceptionInstance_Class(ptr))
                },
                pvalue: unsafe { Py::from_borrowed_ptr(obj.py(), obj.as_ptr()) },
                ptraceback: None,
            })
        } else if unsafe { ffi::PyExceptionClass_Check(obj.as_ptr()) } != 0 {
            PyErrState::FfiTuple {
                ptype: unsafe { Some(Py::from_borrowed_ptr(obj.py(), ptr)) },
                pvalue: None,
                ptraceback: None,
            }
        } else {
            return exceptions_must_derive_from_base_exception(obj.py());
        };

        PyErr::from_state(state)
    }

    /// Get the type of this exception object.
    ///
    /// The object will be normalized first if needed.
    ///
    /// # Examples
    /// ```rust
    /// use pyo3::{Python, PyErr, exceptions::PyTypeError, types::PyType};
    /// Python::with_gil(|py| {
    ///     let err = PyTypeError::new_err(("some type error",));
    ///     assert_eq!(err.ptype(py), PyType::new::<PyTypeError>(py));
    /// });
    /// ```
    pub fn ptype<'py>(&'py self, py: Python<'py>) -> &'py PyType {
        self.normalized(py).ptype.as_ref(py)
    }

    /// Get the value of this exception object.
    ///
    /// The object will be normalized first if needed.
    ///
    /// # Examples
    /// ```rust
    /// use pyo3::{Python, PyErr, exceptions::PyTypeError, types::PyType};
    /// Python::with_gil(|py| {
    ///     let err = PyTypeError::new_err(("some type error",));
    ///     assert!(err.is_instance::<PyTypeError>(py));
    ///     assert_eq!(err.pvalue(py).to_string(), "some type error");
    /// });
    /// ```
    pub fn pvalue<'py>(&'py self, py: Python<'py>) -> &'py PyBaseException {
        self.normalized(py).pvalue.as_ref(py)
    }

    /// Get the value of this exception object.
    ///
    /// The object will be normalized first if needed.
    ///
    /// # Examples
    /// ```rust
    /// use pyo3::{Python, PyErr, exceptions::PyTypeError, types::PyType};
    /// Python::with_gil(|py| {
    ///     let err = PyTypeError::new_err(("some type error",));
    ///     assert_eq!(err.ptraceback(py), None);
    /// });
    /// ```
    pub fn ptraceback<'py>(&'py self, py: Python<'py>) -> Option<&'py PyAny> {
        self.normalized(py)
            .ptraceback
            .as_ref()
            .map(|obj| obj.as_ref(py))
    }

    /// Gets whether an error is present in the Python interpreter's global state.
    #[inline]
    pub fn occurred(_: Python) -> bool {
        unsafe { !ffi::PyErr_Occurred().is_null() }
    }

    /// Retrieves the current error from the Python interpreter's global state.
    ///
    /// The error is cleared from the Python interpreter.
    /// If no error is set, returns a `None`.
    ///
    /// If the error fetched is a `PanicException` (which would have originated from a panic in a
    /// pyo3 callback) then this function will resume the panic.
    pub fn fetch(py: Python) -> Option<PyErr> {
        unsafe {
            let mut ptype: *mut ffi::PyObject = std::ptr::null_mut();
            let mut pvalue: *mut ffi::PyObject = std::ptr::null_mut();
            let mut ptraceback: *mut ffi::PyObject = std::ptr::null_mut();
            ffi::PyErr_Fetch(&mut ptype, &mut pvalue, &mut ptraceback);

            // If the error indicator is not set, all three variables are set to NULL
            if ptype.is_null() && pvalue.is_null() && ptraceback.is_null() {
                return None;
            }

            let err = PyErr::new_from_ffi_tuple(py, ptype, pvalue, ptraceback);

            if ptype == PanicException::type_object(py).as_ptr() {
                let msg: String = PyAny::from_borrowed_ptr_or_opt(py, pvalue)
                    .and_then(|obj| obj.extract().ok())
                    .unwrap_or_else(|| String::from("Unwrapped panic from Python code"));

                eprintln!(
                    "--- PyO3 is resuming a panic after fetching a PanicException from Python. ---"
                );
                eprintln!("Python stack trace below:");
                err.print(py);

                std::panic::resume_unwind(Box::new(msg))
            }

            Some(err)
        }
    }

    /// Retrieves the current error from the Python interpreter's global state.
    ///
    /// The error is cleared from the Python interpreter.
    /// If no error is set, returns a `SystemError` in release mode,
    /// panics in debug mode.
    pub(crate) fn api_call_failed(py: Python) -> PyErr {
        #[cfg(debug_assertions)]
        {
            PyErr::fetch(py).expect("error return without exception set")
        }
        #[cfg(not(debug_assertions))]
        {
            use crate::exceptions::PySystemError;

            PyErr::fetch(py)
                .unwrap_or_else(|| PySystemError::new_err("error return without exception set"))
        }
    }

    /// Creates a new exception type with the given name, which must be of the form
    /// `<module>.<ExceptionName>`, as required by `PyErr_NewException`.
    ///
    /// `base` can be an existing exception type to subclass, or a tuple of classes
    /// `dict` specifies an optional dictionary of class variables and methods
    pub fn new_type<'p>(
        _: Python<'p>,
        name: &str,
        base: Option<&PyType>,
        dict: Option<PyObject>,
    ) -> NonNull<ffi::PyTypeObject> {
        let base: *mut ffi::PyObject = match base {
            None => std::ptr::null_mut(),
            Some(obj) => obj.as_ptr(),
        };

        let dict: *mut ffi::PyObject = match dict {
            None => std::ptr::null_mut(),
            Some(obj) => obj.as_ptr(),
        };

        unsafe {
            let null_terminated_name =
                CString::new(name).expect("Failed to initialize nul terminated exception name");

            NonNull::new_unchecked(ffi::PyErr_NewException(
                null_terminated_name.as_ptr() as *mut c_char,
                base,
                dict,
            ) as *mut ffi::PyTypeObject)
        }
    }

    /// Create a PyErr from an ffi tuple
    ///
    /// # Safety
    /// - `ptype` must be a pointer to valid Python exception type object.
    /// - `pvalue` must be a pointer to a valid Python object, or NULL.
    /// - `ptraceback` must be a pointer to a valid Python traceback object, or NULL.
    unsafe fn new_from_ffi_tuple(
        py: Python,
        ptype: *mut ffi::PyObject,
        pvalue: *mut ffi::PyObject,
        ptraceback: *mut ffi::PyObject,
    ) -> PyErr {
        // Note: must not panic to ensure all owned pointers get acquired correctly,
        // and because we mustn't panic in normalize().
        PyErr::from_state(PyErrState::FfiTuple {
            ptype: Py::from_owned_ptr_or_opt(py, ptype),
            pvalue: Py::from_owned_ptr_or_opt(py, pvalue),
            ptraceback: Py::from_owned_ptr_or_opt(py, ptraceback),
        })
    }

    /// Prints a standard traceback to `sys.stderr`.
    pub fn print(&self, py: Python) {
        self.clone_ref(py).restore(py);
        unsafe { ffi::PyErr_PrintEx(0) }
    }

    /// Prints a standard traceback to `sys.stderr`, and sets
    /// `sys.last_{type,value,traceback}` attributes to this exception's data.
    pub fn print_and_set_sys_last_vars(&self, py: Python) {
        self.clone_ref(py).restore(py);
        unsafe { ffi::PyErr_PrintEx(1) }
    }

    /// Returns true if the current exception matches the exception in `exc`.
    ///
    /// If `exc` is a class object, this also returns `true` when `self` is an instance of a subclass.
    /// If `exc` is a tuple, all exceptions in the tuple (and recursively in subtuples) are searched for a match.
    pub fn matches<T>(&self, py: Python, exc: T) -> bool
    where
        T: ToBorrowedObject,
    {
        exc.with_borrowed_ptr(py, |exc| unsafe {
            ffi::PyErr_GivenExceptionMatches(self.ptype_ptr(), exc) != 0
        })
    }

    /// Returns true if the current exception is instance of `T`.
    pub fn is_instance<T>(&self, py: Python) -> bool
    where
        T: PyTypeObject,
    {
        unsafe {
            ffi::PyErr_GivenExceptionMatches(self.ptype_ptr(), T::type_object(py).as_ptr()) != 0
        }
    }

    /// Retrieves the exception instance for this error.
    pub fn instance<'py>(&'py self, py: Python<'py>) -> &'py PyBaseException {
        self.normalized(py).pvalue.as_ref(py)
    }

    /// Consumes self to take ownership of the exception instance for this error.
    pub fn into_instance(self, py: Python) -> Py<PyBaseException> {
        let out = self.normalized(py).pvalue.as_ref(py).into();
        std::mem::forget(self);
        out
    }

    /// Writes the error back to the Python interpreter's global state.
    /// This is the opposite of `PyErr::fetch()`.
    #[inline]
    pub fn restore(self, py: Python) {
        let (ptype, pvalue, ptraceback) = self
            .state
            .into_inner()
            .expect("Cannot restore a PyErr while normalizing it")
            .into_ffi_tuple(py);
        unsafe { ffi::PyErr_Restore(ptype, pvalue, ptraceback) }
    }

    /// Issues a warning message.
    /// May return a `PyErr` if warnings-as-errors is enabled.
    pub fn warn(py: Python, category: &PyAny, message: &str, stacklevel: i32) -> PyResult<()> {
        let message = CString::new(message)?;
        unsafe {
            error_on_minusone(
                py,
                ffi::PyErr_WarnEx(
                    category.as_ptr(),
                    message.as_ptr(),
                    stacklevel as ffi::Py_ssize_t,
                ),
            )
        }
    }

    /// Clone the PyErr. This requires the GIL, which is why PyErr does not implement Clone.
    ///
    /// # Examples
    /// ```rust
    /// use pyo3::{Python, PyErr, exceptions::PyTypeError, types::PyType};
    /// Python::with_gil(|py| {
    ///     let err = PyTypeError::new_err(("some type error",));
    ///     let err_clone = err.clone_ref(py);
    ///     assert_eq!(err.ptype(py), err_clone.ptype(py));
    ///     assert_eq!(err.pvalue(py), err_clone.pvalue(py));
    ///     assert_eq!(err.ptraceback(py), err_clone.ptraceback(py));
    /// });
    /// ```
    pub fn clone_ref(&self, py: Python) -> PyErr {
        PyErr::from_state(PyErrState::Normalized(self.normalized(py).clone()))
    }

    /// Return the cause (either an exception instance, or None, set by `raise ... from ...`)
    /// associated with the exception, as accessible from Python through `__cause__`.
    pub fn cause(&self, py: Python) -> Option<PyErr> {
        let ptr = unsafe { ffi::PyException_GetCause(self.pvalue(py).as_ptr()) };
        let obj = unsafe { py.from_owned_ptr_or_opt::<PyAny>(ptr) };
        obj.map(|x| Self::from_instance(x))
    }

    /// Set the cause associated with the exception, pass `None` to clear it.
    pub fn set_cause(&self, py: Python, cause: Option<Self>) {
        if let Some(cause) = cause {
            let cause = cause.into_instance(py);
            unsafe {
                ffi::PyException_SetCause(self.pvalue(py).as_ptr(), cause.as_ptr());
            }
        } else {
            unsafe {
                ffi::PyException_SetCause(self.pvalue(py).as_ptr(), std::ptr::null_mut());
            }
        }
    }

    fn from_state(state: PyErrState) -> PyErr {
        PyErr {
            state: UnsafeCell::new(Some(state)),
        }
    }

    /// Returns borrowed reference to this Err's type
    fn ptype_ptr(&self) -> *mut ffi::PyObject {
        match unsafe { &*self.state.get() } {
            Some(PyErrState::Lazy { ptype, .. }) => ptype.as_ptr(),
            Some(PyErrState::FfiTuple { ptype, .. }) => ptype.as_ptr(),
            Some(PyErrState::Normalized(n)) => n.ptype.as_ptr(),
            None => panic!("Cannot access exception type while normalizing"),
        }
    }

    fn normalized(&self, py: Python) -> &PyErrStateNormalized {
        // This process is safe because:
        // - Access is guaranteed not to be concurrent thanks to `Python` GIL token
        // - Write happens only once, and then never will change again.
        // - State is set to None during the normalization process, so that a second
        //   concurrent normalization attempt will panic before changing anything.

        if let Some(PyErrState::Normalized(n)) = unsafe { &*self.state.get() } {
            return n;
        }

        let state = unsafe {
            (*self.state.get())
                .take()
                .expect("Cannot normalize a PyErr while already normalizing it.")
        };
        let (mut ptype, mut pvalue, mut ptraceback) = state.into_ffi_tuple(py);

        unsafe {
            ffi::PyErr_NormalizeException(&mut ptype, &mut pvalue, &mut ptraceback);
            let self_state = &mut *self.state.get();
            *self_state = Some(PyErrState::Normalized(PyErrStateNormalized {
                ptype: Py::from_owned_ptr_or_opt(py, ptype).expect("Exception type missing"),
                pvalue: Py::from_owned_ptr_or_opt(py, pvalue).expect("Exception value missing"),
                ptraceback: PyObject::from_owned_ptr_or_opt(py, ptraceback),
            }));

            match self_state {
                Some(PyErrState::Normalized(n)) => n,
                _ => unreachable!(),
            }
        }
    }
}

impl std::fmt::Debug for PyErr {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        Python::with_gil(|py| {
            f.debug_struct("PyErr")
                .field("type", self.ptype(py))
                .field("value", self.pvalue(py))
                .field("traceback", &self.ptraceback(py))
                .finish()
        })
    }
}

impl std::fmt::Display for PyErr {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        Python::with_gil(|py| {
            let instance = self.instance(py);
            let type_name = instance.get_type().name().map_err(|_| std::fmt::Error)?;
            write!(f, "{}", type_name)?;
            if let Ok(s) = instance.str() {
                write!(f, ": {}", &s.to_string_lossy())
            } else {
                write!(f, ": <exception str() failed>")
            }
        })
    }
}

impl std::error::Error for PyErr {}

impl IntoPy<PyObject> for PyErr {
    fn into_py(self, py: Python) -> PyObject {
        self.into_instance(py).into()
    }
}

impl ToPyObject for PyErr {
    fn to_object(&self, py: Python) -> PyObject {
        self.clone_ref(py).into_py(py)
    }
}

impl<'a> IntoPy<PyObject> for &'a PyErr {
    fn into_py(self, py: Python) -> PyObject {
        self.clone_ref(py).into_py(py)
    }
}

/// Convert `PyDowncastError` to Python `TypeError`.
impl<'a> std::convert::From<PyDowncastError<'a>> for PyErr {
    fn from(err: PyDowncastError) -> PyErr {
        exceptions::PyTypeError::new_err(err.to_string())
    }
}

impl<'a> std::error::Error for PyDowncastError<'a> {}

impl<'a> std::fmt::Display for PyDowncastError<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        write!(
            f,
            "'{}' object cannot be converted to '{}'",
            self.from.get_type().name().map_err(|_| std::fmt::Error)?,
            self.to
        )
    }
}

pub fn panic_after_error(_py: Python) -> ! {
    unsafe {
        ffi::PyErr_Print();
    }
    panic!("Python API call failed");
}

/// Returns Ok if the error code is not -1.
#[inline]
pub fn error_on_minusone(py: Python, result: c_int) -> PyResult<()> {
    if result != -1 {
        Ok(())
    } else {
        Err(PyErr::api_call_failed(py))
    }
}

#[inline]
fn exceptions_must_derive_from_base_exception(py: Python) -> PyErr {
    PyErr::from_state(PyErrState::Lazy {
        ptype: exceptions::PyTypeError::type_object(py).into(),
        pvalue: boxed_args("exceptions must derive from BaseException"),
    })
}

#[cfg(test)]
mod tests {
    use super::PyErrState;
    use crate::exceptions;
    use crate::{PyErr, Python};

    #[test]
    fn no_error() {
        Python::with_gil(|py| {
            assert!(PyErr::fetch(py).is_none());
        });
    }

    #[test]
    fn set_typeerror() {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let err: PyErr = exceptions::PyTypeError::new_err(());
        err.restore(py);
        assert!(PyErr::occurred(py));
        drop(PyErr::fetch(py));
    }

    #[test]
    #[should_panic(expected = "new panic")]
    fn fetching_panic_exception_resumes_unwind() {
        use crate::panic::PanicException;

        let gil = Python::acquire_gil();
        let py = gil.python();
        let err: PyErr = PanicException::new_err("new panic");
        err.restore(py);
        assert!(PyErr::occurred(py));

        // should resume unwind
        let _ = PyErr::fetch(py);
    }

    #[test]
    fn err_debug() {
        // Debug representation should be like the following (without the newlines):
        // PyErr {
        //     type: <class 'Exception'>,
        //     value: Exception('banana'),
        //     traceback: Some(<traceback object at 0x..)"
        // }

        let gil = Python::acquire_gil();
        let py = gil.python();
        let err = py
            .run("raise Exception('banana')", None, None)
            .expect_err("raising should have given us an error");

        let debug_str = format!("{:?}", err);
        assert!(debug_str.starts_with("PyErr { "));
        assert!(debug_str.ends_with(" }"));

        // strip "PyErr { " and " }"
        let mut fields = debug_str["PyErr { ".len()..debug_str.len() - 2].split(", ");

        assert_eq!(fields.next().unwrap(), "type: <class 'Exception'>");
        if py.version_info() >= (3, 7) {
            assert_eq!(fields.next().unwrap(), "value: Exception('banana')");
        } else {
            // Python 3.6 and below formats the repr differently
            assert_eq!(fields.next().unwrap(), ("value: Exception('banana',)"));
        }

        let traceback = fields.next().unwrap();
        assert!(traceback.starts_with("traceback: Some(<traceback object at 0x"));
        assert!(traceback.ends_with(">)"));

        assert!(fields.next().is_none());
    }

    #[test]
    fn err_display() {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let err = py
            .run("raise Exception('banana')", None, None)
            .expect_err("raising should have given us an error");
        assert_eq!(err.to_string(), "Exception: banana");
    }

    #[test]
    fn test_pyerr_send_sync() {
        fn is_send<T: Send>() {}
        fn is_sync<T: Sync>() {}

        is_send::<PyErr>();
        is_sync::<PyErr>();

        is_send::<PyErrState>();
        is_sync::<PyErrState>();
    }

    #[test]
    fn test_pyerr_cause() {
        Python::with_gil(|py| {
            let err = py
                .run("raise Exception('banana')", None, None)
                .expect_err("raising should have given us an error");
            assert!(err.cause(py).is_none());

            let err = py
                .run(
                    "raise Exception('banana') from Exception('apple')",
                    None,
                    None,
                )
                .expect_err("raising should have given us an error");
            let cause = err
                .cause(py)
                .expect("raising from should have given us a cause");
            assert_eq!(cause.to_string(), "Exception: apple");

            err.set_cause(py, None);
            assert!(err.cause(py).is_none());

            let new_cause = exceptions::PyValueError::new_err("orange");
            err.set_cause(py, Some(new_cause));
            let cause = err
                .cause(py)
                .expect("set_cause should have given us a cause");
            assert_eq!(cause.to_string(), "ValueError: orange");
        });
    }
}
