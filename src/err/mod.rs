use crate::conversion::IntoPyObject;
use crate::ffi_ptr_ext::FfiPtrExt;
#[cfg(feature = "experimental-inspect")]
use crate::inspect::PyStaticExpr;
use crate::instance::Bound;
#[cfg(Py_3_11)]
use crate::intern;
use crate::panic::PanicException;
use crate::py_result_ext::PyResultExt;
use crate::type_object::PyTypeInfo;
use crate::types::any::PyAnyMethods;
#[cfg(Py_3_11)]
use crate::types::PyString;
use crate::types::{
    string::PyStringMethods, traceback::PyTracebackMethods, typeobject::PyTypeMethods, PyTraceback,
    PyType,
};
use crate::{exceptions::PyBaseException, ffi};
use crate::{BoundObject, Py, PyAny, Python};
use err_state::{PyErrState, PyErrStateLazyFnOutput, PyErrStateNormalized};
use std::convert::Infallible;
use std::ffi::CStr;

mod cast_error;
mod downcast_error;
mod err_state;
mod impls;

pub use cast_error::{CastError, CastIntoError};
#[allow(deprecated)]
pub use downcast_error::{DowncastError, DowncastIntoError};

/// Represents a Python exception.
///
/// To avoid needing access to [`Python`] in `Into` conversions to create `PyErr` (thus improving
/// compatibility with `?` and other Rust errors) this type supports creating exceptions instances
/// in a lazy fashion, where the full Python object for the exception is created only when needed.
///
/// Accessing the contained exception in any way, such as with [`value`](PyErr::value),
/// [`get_type`](PyErr::get_type), or [`is_instance`](PyErr::is_instance)
/// will create the full exception object if it was not already created.
pub struct PyErr {
    state: PyErrState,
}

// The inner value is only accessed through ways that require proving the gil is held
#[cfg(feature = "nightly")]
unsafe impl crate::marker::Ungil for PyErr {}

/// Represents the result of a Python call.
pub type PyResult<T> = Result<T, PyErr>;

/// Helper conversion trait that allows to use custom arguments for lazy exception construction.
pub trait PyErrArguments: Send + Sync {
    /// Arguments for exception
    fn arguments(self, py: Python<'_>) -> Py<PyAny>;
}

impl<T> PyErrArguments for T
where
    T: for<'py> IntoPyObject<'py> + Send + Sync,
{
    fn arguments(self, py: Python<'_>) -> Py<PyAny> {
        // FIXME: `arguments` should become fallible
        match self.into_pyobject(py) {
            Ok(obj) => obj.into_any().unbind(),
            Err(e) => panic!("Converting PyErr arguments failed: {}", e.into()),
        }
    }
}

impl PyErr {
    /// Creates a new PyErr of type `T`.
    ///
    /// `args` can be:
    /// * a tuple: the exception instance will be created using the equivalent to the Python
    ///   expression `T(*tuple)`
    /// * any other value: the exception instance will be created using the equivalent to the Python
    ///   expression `T(value)`
    ///
    /// This exception instance will be initialized lazily. This avoids the need for the Python GIL
    /// to be held, but requires `args` to be `Send` and `Sync`. If `args` is not `Send` or `Sync`,
    /// consider using [`PyErr::from_value`] instead.
    ///
    /// If `T` does not inherit from `BaseException`, then a `TypeError` will be returned.
    ///
    /// If calling T's constructor with `args` raises an exception, that exception will be returned.
    ///
    /// # Examples
    ///
    /// ```
    /// use pyo3::prelude::*;
    /// use pyo3::exceptions::PyTypeError;
    ///
    /// #[pyfunction]
    /// fn always_throws() -> PyResult<()> {
    ///     Err(PyErr::new::<PyTypeError, _>("Error message"))
    /// }
    /// #
    /// # Python::attach(|py| {
    /// #     let fun = pyo3::wrap_pyfunction!(always_throws, py).unwrap();
    /// #     let err = fun.call0().expect_err("called a function that should always return an error but the return value was Ok");
    /// #     assert!(err.is_instance_of::<PyTypeError>(py))
    /// # });
    /// ```
    ///
    /// In most cases, you can use a concrete exception's constructor instead:
    ///
    /// ```
    /// use pyo3::prelude::*;
    /// use pyo3::exceptions::PyTypeError;
    ///
    /// #[pyfunction]
    /// fn always_throws() -> PyResult<()> {
    ///     Err(PyTypeError::new_err("Error message"))
    /// }
    /// #
    /// # Python::attach(|py| {
    /// #     let fun = pyo3::wrap_pyfunction!(always_throws, py).unwrap();
    /// #     let err = fun.call0().expect_err("called a function that should always return an error but the return value was Ok");
    /// #     assert!(err.is_instance_of::<PyTypeError>(py))
    /// # });
    /// ```
    #[inline]
    pub fn new<T, A>(args: A) -> PyErr
    where
        T: PyTypeInfo,
        A: PyErrArguments + Send + Sync + 'static,
    {
        PyErr::from_state(PyErrState::lazy(Box::new(move |py| {
            PyErrStateLazyFnOutput {
                ptype: T::type_object(py).into(),
                pvalue: args.arguments(py),
            }
        })))
    }

    /// Constructs a new PyErr from the given Python type and arguments.
    ///
    /// `ty` is the exception type; usually one of the standard exceptions
    /// like `exceptions::PyRuntimeError`.
    ///
    /// `args` is either a tuple or a single value, with the same meaning as in [`PyErr::new`].
    ///
    /// If `ty` does not inherit from `BaseException`, then a `TypeError` will be returned.
    ///
    /// If calling `ty` with `args` raises an exception, that exception will be returned.
    pub fn from_type<A>(ty: Bound<'_, PyType>, args: A) -> PyErr
    where
        A: PyErrArguments + Send + Sync + 'static,
    {
        PyErr::from_state(PyErrState::lazy_arguments(ty.unbind().into_any(), args))
    }

    /// Creates a new PyErr.
    ///
    /// If `obj` is a Python exception object, the PyErr will contain that object.
    ///
    /// If `obj` is a Python exception type object, this is equivalent to `PyErr::from_type(obj, ())`.
    ///
    /// Otherwise, a `TypeError` is created.
    ///
    /// # Examples
    /// ```rust
    /// use pyo3::prelude::*;
    /// use pyo3::PyTypeInfo;
    /// use pyo3::exceptions::PyTypeError;
    /// use pyo3::types::PyString;
    ///
    /// Python::attach(|py| {
    ///     // Case #1: Exception object
    ///     let err = PyErr::from_value(PyTypeError::new_err("some type error")
    ///         .value(py).clone().into_any());
    ///     assert_eq!(err.to_string(), "TypeError: some type error");
    ///
    ///     // Case #2: Exception type
    ///     let err = PyErr::from_value(PyTypeError::type_object(py).into_any());
    ///     assert_eq!(err.to_string(), "TypeError: ");
    ///
    ///     // Case #3: Invalid exception value
    ///     let err = PyErr::from_value(PyString::new(py, "foo").into_any());
    ///     assert_eq!(
    ///         err.to_string(),
    ///         "TypeError: exceptions must derive from BaseException"
    ///     );
    /// });
    /// ```
    pub fn from_value(obj: Bound<'_, PyAny>) -> PyErr {
        let state = match obj.cast_into::<PyBaseException>() {
            Ok(obj) => PyErrState::normalized(PyErrStateNormalized::new(obj)),
            Err(err) => {
                // Assume obj is Type[Exception]; let later normalization handle if this
                // is not the case
                let obj = err.into_inner();
                let py = obj.py();
                PyErrState::lazy_arguments(obj.unbind(), py.None())
            }
        };

        PyErr::from_state(state)
    }

    /// Returns the type of this exception.
    ///
    /// # Examples
    /// ```rust
    /// use pyo3::{prelude::*, exceptions::PyTypeError, types::PyType};
    ///
    /// Python::attach(|py| {
    ///     let err: PyErr = PyTypeError::new_err(("some type error",));
    ///     assert!(err.get_type(py).is(&PyType::new::<PyTypeError>(py)));
    /// });
    /// ```
    pub fn get_type<'py>(&self, py: Python<'py>) -> Bound<'py, PyType> {
        self.normalized(py).ptype(py)
    }

    /// Returns the value of this exception.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use pyo3::{exceptions::PyTypeError, PyErr, Python};
    ///
    /// Python::attach(|py| {
    ///     let err: PyErr = PyTypeError::new_err(("some type error",));
    ///     assert!(err.is_instance_of::<PyTypeError>(py));
    ///     assert_eq!(err.value(py).to_string(), "some type error");
    /// });
    /// ```
    pub fn value<'py>(&self, py: Python<'py>) -> &Bound<'py, PyBaseException> {
        self.normalized(py).pvalue.bind(py)
    }

    /// Consumes self to take ownership of the exception value contained in this error.
    pub fn into_value(self, py: Python<'_>) -> Py<PyBaseException> {
        // NB technically this causes one reference count increase and decrease in quick succession
        // on pvalue, but it's probably not worth optimizing this right now for the additional code
        // complexity.
        let normalized = self.normalized(py);
        let exc = normalized.pvalue.clone_ref(py);
        if let Some(tb) = normalized.ptraceback(py) {
            unsafe {
                ffi::PyException_SetTraceback(exc.as_ptr(), tb.as_ptr());
            }
        }
        exc
    }

    /// Returns the traceback of this exception object.
    ///
    /// # Examples
    /// ```rust
    /// use pyo3::{exceptions::PyTypeError, Python};
    ///
    /// Python::attach(|py| {
    ///     let err = PyTypeError::new_err(("some type error",));
    ///     assert!(err.traceback(py).is_none());
    /// });
    /// ```
    pub fn traceback<'py>(&self, py: Python<'py>) -> Option<Bound<'py, PyTraceback>> {
        self.normalized(py).ptraceback(py)
    }

    /// Gets whether an error is present in the Python interpreter's global state.
    #[inline]
    pub fn occurred(_: Python<'_>) -> bool {
        unsafe { !ffi::PyErr_Occurred().is_null() }
    }

    /// Takes the current error from the Python interpreter's global state and clears the global
    /// state. If no error is set, returns `None`.
    ///
    /// If the error is a `PanicException` (which would have originated from a panic in a pyo3
    /// callback) then this function will resume the panic.
    ///
    /// Use this function when it is not known if an error should be present. If the error is
    /// expected to have been set, for example from [`PyErr::occurred`] or by an error return value
    /// from a C FFI function, use [`PyErr::fetch`].
    pub fn take(py: Python<'_>) -> Option<PyErr> {
        let state = PyErrStateNormalized::take(py)?;

        if PanicException::is_exact_type_of(state.pvalue.bind(py)) {
            Self::print_panic_and_unwind(py, state)
        }

        Some(PyErr::from_state(PyErrState::normalized(state)))
    }

    #[cold]
    fn print_panic_and_unwind(py: Python<'_>, state: PyErrStateNormalized) -> ! {
        let msg: String = state
            .pvalue
            .bind(py)
            .str()
            .map(|py_str| py_str.to_string_lossy().into())
            .unwrap_or_else(|_| String::from("Unwrapped panic from Python code"));

        eprintln!("--- PyO3 is resuming a panic after fetching a PanicException from Python. ---");
        eprintln!("Python stack trace below:");

        PyErrState::normalized(state).restore(py);

        // SAFETY: thread is attached and error was just set in the interpreter
        unsafe {
            ffi::PyErr_PrintEx(0);
        }

        std::panic::resume_unwind(Box::new(msg))
    }

    /// Equivalent to [PyErr::take], but when no error is set:
    ///  - Panics in debug mode.
    ///  - Returns a `SystemError` in release mode.
    ///
    /// This behavior is consistent with Python's internal handling of what happens when a C return
    /// value indicates an error occurred but the global error state is empty. (A lack of exception
    /// should be treated as a bug in the code which returned an error code but did not set an
    /// exception.)
    ///
    /// Use this function when the error is expected to have been set, for example from
    /// [PyErr::occurred] or by an error return value from a C FFI function.
    #[cfg_attr(debug_assertions, track_caller)]
    #[inline]
    pub fn fetch(py: Python<'_>) -> PyErr {
        PyErr::take(py).unwrap_or_else(failed_to_fetch)
    }

    /// Creates a new exception type with the given name and docstring.
    ///
    /// - `base` can be an existing exception type to subclass, or a tuple of classes.
    /// - `dict` specifies an optional dictionary of class variables and methods.
    /// - `doc` will be the docstring seen by python users.
    ///
    ///
    /// # Errors
    ///
    /// This function returns an error if `name` is not of the form `<module>.<ExceptionName>`.
    pub fn new_type<'py>(
        py: Python<'py>,
        name: &CStr,
        doc: Option<&CStr>,
        base: Option<&Bound<'py, PyType>>,
        dict: Option<Py<PyAny>>,
    ) -> PyResult<Py<PyType>> {
        let base: *mut ffi::PyObject = match base {
            None => std::ptr::null_mut(),
            Some(obj) => obj.as_ptr(),
        };

        let dict: *mut ffi::PyObject = match dict {
            None => std::ptr::null_mut(),
            Some(obj) => obj.as_ptr(),
        };

        let doc_ptr = match doc.as_ref() {
            Some(c) => c.as_ptr(),
            None => std::ptr::null(),
        };

        // SAFETY: correct call to FFI function, return value is known to be a new
        // exception type or null on error
        unsafe {
            ffi::PyErr_NewExceptionWithDoc(name.as_ptr(), doc_ptr, base, dict)
                .assume_owned_or_err(py)
                .cast_into_unchecked()
        }
        .map(Bound::unbind)
    }

    /// Prints a standard traceback to `sys.stderr`.
    pub fn display(&self, py: Python<'_>) {
        #[cfg(Py_3_12)]
        unsafe {
            ffi::PyErr_DisplayException(self.value(py).as_ptr())
        }

        #[cfg(not(Py_3_12))]
        unsafe {
            // keep the bound `traceback` alive for entire duration of
            // PyErr_Display. if we inline this, the `Bound` will be dropped
            // after the argument got evaluated, leading to call with a dangling
            // pointer.
            let traceback = self.traceback(py);
            let type_bound = self.get_type(py);
            ffi::PyErr_Display(
                type_bound.as_ptr(),
                self.value(py).as_ptr(),
                traceback
                    .as_ref()
                    .map_or(std::ptr::null_mut(), |traceback| traceback.as_ptr()),
            )
        }
    }

    /// Calls `sys.excepthook` and then prints a standard traceback to `sys.stderr`.
    pub fn print(&self, py: Python<'_>) {
        self.clone_ref(py).restore(py);
        unsafe { ffi::PyErr_PrintEx(0) }
    }

    /// Calls `sys.excepthook` and then prints a standard traceback to `sys.stderr`.
    ///
    /// Additionally sets `sys.last_{type,value,traceback,exc}` attributes to this exception.
    pub fn print_and_set_sys_last_vars(&self, py: Python<'_>) {
        self.clone_ref(py).restore(py);
        unsafe { ffi::PyErr_PrintEx(1) }
    }

    /// Returns true if the current exception matches the exception in `exc`.
    ///
    /// If `exc` is a class object, this also returns `true` when `self` is an instance of a subclass.
    /// If `exc` is a tuple, all exceptions in the tuple (and recursively in subtuples) are searched for a match.
    pub fn matches<'py, T>(&self, py: Python<'py>, exc: T) -> Result<bool, T::Error>
    where
        T: IntoPyObject<'py>,
    {
        Ok(self.is_instance(py, &exc.into_pyobject(py)?.into_any().as_borrowed()))
    }

    /// Returns true if the current exception is instance of `T`.
    #[inline]
    pub fn is_instance(&self, py: Python<'_>, ty: &Bound<'_, PyAny>) -> bool {
        let type_bound = self.get_type(py);
        (unsafe { ffi::PyErr_GivenExceptionMatches(type_bound.as_ptr(), ty.as_ptr()) }) != 0
    }

    /// Returns true if the current exception is instance of `T`.
    #[inline]
    pub fn is_instance_of<T>(&self, py: Python<'_>) -> bool
    where
        T: PyTypeInfo,
    {
        self.is_instance(py, &T::type_object(py))
    }

    /// Writes the error back to the Python interpreter's global state.
    /// This is the opposite of `PyErr::fetch()`.
    #[inline]
    pub fn restore(self, py: Python<'_>) {
        self.state.restore(py)
    }

    /// Reports the error as unraisable.
    ///
    /// This calls `sys.unraisablehook()` using the current exception and obj argument.
    ///
    /// This method is useful to report errors in situations where there is no good mechanism
    /// to report back to the Python land.  In Python this is used to indicate errors in
    /// background threads or destructors which are protected.  In Rust code this is commonly
    /// useful when you are calling into a Python callback which might fail, but there is no
    /// obvious way to handle this error other than logging it.
    ///
    /// Calling this method has the benefit that the error goes back into a standardized callback
    /// in Python which for instance allows unittests to ensure that no unraisable error
    /// actually happened by hooking `sys.unraisablehook`.
    ///
    /// Example:
    /// ```rust
    /// # use pyo3::prelude::*;
    /// # use pyo3::exceptions::PyRuntimeError;
    /// # fn failing_function() -> PyResult<()> { Err(PyRuntimeError::new_err("foo")) }
    /// # fn main() -> PyResult<()> {
    /// Python::attach(|py| {
    ///     match failing_function() {
    ///         Err(pyerr) => pyerr.write_unraisable(py, None),
    ///         Ok(..) => { /* do something here */ }
    ///     }
    ///     Ok(())
    /// })
    /// # }
    #[inline]
    pub fn write_unraisable(self, py: Python<'_>, obj: Option<&Bound<'_, PyAny>>) {
        self.restore(py);
        unsafe { ffi::PyErr_WriteUnraisable(obj.map_or(std::ptr::null_mut(), Bound::as_ptr)) }
    }

    /// Issues a warning message.
    ///
    /// May return an `Err(PyErr)` if warnings-as-errors is enabled.
    ///
    /// Equivalent to `warnings.warn()` in Python.
    ///
    /// The `category` should be one of the `Warning` classes available in
    /// [`pyo3::exceptions`](crate::exceptions), or a subclass.  The Python
    /// object can be retrieved using [`Python::get_type()`].
    ///
    /// Example:
    /// ```rust
    /// # use pyo3::prelude::*;
    /// # use pyo3::ffi::c_str;
    /// # fn main() -> PyResult<()> {
    /// Python::attach(|py| {
    ///     let user_warning = py.get_type::<pyo3::exceptions::PyUserWarning>();
    ///     PyErr::warn(py, &user_warning, c"I am warning you", 0)?;
    ///     Ok(())
    /// })
    /// # }
    /// ```
    pub fn warn<'py>(
        py: Python<'py>,
        category: &Bound<'py, PyAny>,
        message: &CStr,
        stacklevel: i32,
    ) -> PyResult<()> {
        error_on_minusone(py, unsafe {
            ffi::PyErr_WarnEx(
                category.as_ptr(),
                message.as_ptr(),
                stacklevel as ffi::Py_ssize_t,
            )
        })
    }

    /// Issues a warning message, with more control over the warning attributes.
    ///
    /// May return a `PyErr` if warnings-as-errors is enabled.
    ///
    /// Equivalent to `warnings.warn_explicit()` in Python.
    ///
    /// The `category` should be one of the `Warning` classes available in
    /// [`pyo3::exceptions`](crate::exceptions), or a subclass.
    pub fn warn_explicit<'py>(
        py: Python<'py>,
        category: &Bound<'py, PyAny>,
        message: &CStr,
        filename: &CStr,
        lineno: i32,
        module: Option<&CStr>,
        registry: Option<&Bound<'py, PyAny>>,
    ) -> PyResult<()> {
        let module_ptr = match module {
            None => std::ptr::null_mut(),
            Some(s) => s.as_ptr(),
        };
        let registry: *mut ffi::PyObject = match registry {
            None => std::ptr::null_mut(),
            Some(obj) => obj.as_ptr(),
        };
        error_on_minusone(py, unsafe {
            ffi::PyErr_WarnExplicit(
                category.as_ptr(),
                message.as_ptr(),
                filename.as_ptr(),
                lineno,
                module_ptr,
                registry,
            )
        })
    }

    /// Clone the PyErr. This requires the GIL, which is why PyErr does not implement Clone.
    ///
    /// # Examples
    /// ```rust
    /// use pyo3::{exceptions::PyTypeError, PyErr, Python, prelude::PyAnyMethods};
    /// Python::attach(|py| {
    ///     let err: PyErr = PyTypeError::new_err(("some type error",));
    ///     let err_clone = err.clone_ref(py);
    ///     assert!(err.get_type(py).is(&err_clone.get_type(py)));
    ///     assert!(err.value(py).is(err_clone.value(py)));
    ///     match err.traceback(py) {
    ///         None => assert!(err_clone.traceback(py).is_none()),
    ///         Some(tb) => assert!(err_clone.traceback(py).unwrap().is(&tb)),
    ///     }
    /// });
    /// ```
    #[inline]
    pub fn clone_ref(&self, py: Python<'_>) -> PyErr {
        PyErr::from_state(PyErrState::normalized(self.normalized(py).clone_ref(py)))
    }

    /// Return the cause (either an exception instance, or None, set by `raise ... from ...`)
    /// associated with the exception, as accessible from Python through `__cause__`.
    pub fn cause(&self, py: Python<'_>) -> Option<PyErr> {
        use crate::ffi_ptr_ext::FfiPtrExt;
        let obj =
            unsafe { ffi::PyException_GetCause(self.value(py).as_ptr()).assume_owned_or_opt(py) };
        // PyException_GetCause is documented as potentially returning PyNone, but only GraalPy seems to actually do that
        #[cfg(GraalPy)]
        if let Some(cause) = &obj {
            if cause.is_none() {
                return None;
            }
        }
        obj.map(Self::from_value)
    }

    /// Set the cause associated with the exception, pass `None` to clear it.
    pub fn set_cause(&self, py: Python<'_>, cause: Option<Self>) {
        let value = self.value(py);
        let cause = cause.map(|err| err.into_value(py));
        unsafe {
            // PyException_SetCause _steals_ a reference to cause, so must use .into_ptr()
            ffi::PyException_SetCause(
                value.as_ptr(),
                cause.map_or(std::ptr::null_mut(), Py::into_ptr),
            );
        }
    }

    /// Equivalent to calling `add_note` on the exception in Python.
    #[cfg(Py_3_11)]
    pub fn add_note<N: for<'py> IntoPyObject<'py, Target = PyString>>(
        &self,
        py: Python<'_>,
        note: N,
    ) -> PyResult<()> {
        self.value(py)
            .call_method1(intern!(py, "add_note"), (note,))?;
        Ok(())
    }

    #[inline]
    fn from_state(state: PyErrState) -> PyErr {
        PyErr { state }
    }

    #[inline]
    fn normalized(&self, py: Python<'_>) -> &PyErrStateNormalized {
        self.state.as_normalized(py)
    }
}

/// Called when `PyErr::fetch` is called but no exception is set.
#[cold]
#[cfg_attr(debug_assertions, track_caller)]
fn failed_to_fetch() -> PyErr {
    const FAILED_TO_FETCH: &str = "attempted to fetch exception but none was set";

    if cfg!(debug_assertions) {
        panic!("{}", FAILED_TO_FETCH)
    } else {
        crate::exceptions::PySystemError::new_err(FAILED_TO_FETCH)
    }
}

impl std::fmt::Debug for PyErr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        Python::attach(|py| {
            f.debug_struct("PyErr")
                .field("type", &self.get_type(py))
                .field("value", self.value(py))
                .field(
                    "traceback",
                    &self.traceback(py).map(|tb| match tb.format() {
                        Ok(s) => s,
                        Err(err) => {
                            err.write_unraisable(py, Some(&tb));
                            // It would be nice to format what we can of the
                            // error, but we can't guarantee that the error
                            // won't have another unformattable traceback inside
                            // it and we want to avoid an infinite recursion.
                            format!("<unformattable {tb:?}>")
                        }
                    }),
                )
                .finish()
        })
    }
}

impl std::fmt::Display for PyErr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Python::attach(|py| {
            let value = self.value(py);
            let type_name = value.get_type().qualname().map_err(|_| std::fmt::Error)?;
            write!(f, "{type_name}")?;
            if let Ok(s) = value.str() {
                write!(f, ": {}", &s.to_string_lossy())
            } else {
                write!(f, ": <exception str() failed>")
            }
        })
    }
}

impl std::error::Error for PyErr {}

impl<'py> IntoPyObject<'py> for PyErr {
    type Target = PyBaseException;
    type Output = Bound<'py, Self::Target>;
    type Error = Infallible;

    #[cfg(feature = "experimental-inspect")]
    const OUTPUT_TYPE: PyStaticExpr = PyBaseException::TYPE_HINT;

    #[inline]
    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        Ok(self.into_value(py).into_bound(py))
    }
}

impl<'py> IntoPyObject<'py> for &PyErr {
    type Target = PyBaseException;
    type Output = Bound<'py, Self::Target>;
    type Error = Infallible;

    #[cfg(feature = "experimental-inspect")]
    const OUTPUT_TYPE: PyStaticExpr = PyErr::OUTPUT_TYPE;

    #[inline]
    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        self.clone_ref(py).into_pyobject(py)
    }
}

/// Python exceptions that can be converted to [`PyErr`].
///
/// This is used to implement [`From<Bound<'_, T>> for PyErr`].
///
/// Users should not need to implement this trait directly. It is implemented automatically in the
/// [`crate::import_exception!`] and [`crate::create_exception!`] macros.
pub trait ToPyErr {}

impl<'py, T> std::convert::From<Bound<'py, T>> for PyErr
where
    T: ToPyErr,
{
    #[inline]
    fn from(err: Bound<'py, T>) -> PyErr {
        PyErr::from_value(err.into_any())
    }
}

/// Returns Ok if the error code is not -1.
#[inline]
pub(crate) fn error_on_minusone<T: SignedInteger>(py: Python<'_>, result: T) -> PyResult<()> {
    if result != T::MINUS_ONE {
        Ok(())
    } else {
        Err(PyErr::fetch(py))
    }
}

pub(crate) trait SignedInteger: Eq {
    const MINUS_ONE: Self;
}

macro_rules! impl_signed_integer {
    ($t:ty) => {
        impl SignedInteger for $t {
            const MINUS_ONE: Self = -1;
        }
    };
}

impl_signed_integer!(i8);
impl_signed_integer!(i16);
impl_signed_integer!(i32);
impl_signed_integer!(i64);
impl_signed_integer!(i128);
impl_signed_integer!(isize);

#[cfg(test)]
mod tests {
    use super::PyErrState;
    use crate::exceptions::{self, PyTypeError, PyValueError};
    use crate::impl_::pyclass::{value_of, IsSend, IsSync};
    use crate::test_utils::assert_warnings;
    use crate::{PyErr, PyTypeInfo, Python};

    #[test]
    fn no_error() {
        assert!(Python::attach(PyErr::take).is_none());
    }

    #[test]
    fn set_valueerror() {
        Python::attach(|py| {
            let err: PyErr = exceptions::PyValueError::new_err("some exception message");
            assert!(err.is_instance_of::<exceptions::PyValueError>(py));
            err.restore(py);
            assert!(PyErr::occurred(py));
            let err = PyErr::fetch(py);
            assert!(err.is_instance_of::<exceptions::PyValueError>(py));
            assert_eq!(err.to_string(), "ValueError: some exception message");
        })
    }

    #[test]
    fn invalid_error_type() {
        Python::attach(|py| {
            let err: PyErr = PyErr::new::<crate::types::PyString, _>(());
            assert!(err.is_instance_of::<exceptions::PyTypeError>(py));
            err.restore(py);
            let err = PyErr::fetch(py);

            assert!(err.is_instance_of::<exceptions::PyTypeError>(py));
            assert_eq!(
                err.to_string(),
                "TypeError: exceptions must derive from BaseException"
            );
        })
    }

    #[test]
    fn set_typeerror() {
        Python::attach(|py| {
            let err: PyErr = exceptions::PyTypeError::new_err(());
            err.restore(py);
            assert!(PyErr::occurred(py));
            drop(PyErr::fetch(py));
        });
    }

    #[test]
    #[should_panic(expected = "new panic")]
    fn fetching_panic_exception_resumes_unwind() {
        use crate::panic::PanicException;

        Python::attach(|py| {
            let err: PyErr = PanicException::new_err("new panic");
            err.restore(py);
            assert!(PyErr::occurred(py));

            // should resume unwind
            let _ = PyErr::fetch(py);
        });
    }

    #[test]
    #[should_panic(expected = "new panic")]
    #[cfg(not(Py_3_12))]
    fn fetching_normalized_panic_exception_resumes_unwind() {
        use crate::panic::PanicException;

        Python::attach(|py| {
            let err: PyErr = PanicException::new_err("new panic");
            // Restoring an error doesn't normalize it before Python 3.12,
            // so we have to explicitly test this case.
            let _ = err.normalized(py);
            err.restore(py);
            assert!(PyErr::occurred(py));

            // should resume unwind
            let _ = PyErr::fetch(py);
        });
    }

    #[test]
    fn err_debug() {
        // Debug representation should be like the following (without the newlines):
        // PyErr {
        //     type: <class 'Exception'>,
        //     value: Exception('banana'),
        //     traceback:  Some(\"Traceback (most recent call last):\\n  File \\\"<string>\\\", line 1, in <module>\\n\")
        // }

        Python::attach(|py| {
            let err = py
                .run(c"raise Exception('banana')", None, None)
                .expect_err("raising should have given us an error");

            let debug_str = format!("{err:?}");
            assert!(debug_str.starts_with("PyErr { "));
            assert!(debug_str.ends_with(" }"));

            // Strip "PyErr { " and " }". Split into 3 substrings to separate type,
            // value, and traceback while not splitting the string within traceback.
            let mut fields = debug_str["PyErr { ".len()..debug_str.len() - 2].splitn(3, ", ");

            assert_eq!(fields.next().unwrap(), "type: <class 'Exception'>");
            assert_eq!(fields.next().unwrap(), "value: Exception('banana')");
            assert_eq!(
                fields.next().unwrap(),
                "traceback: Some(\"Traceback (most recent call last):\\n  File \\\"<string>\\\", line 1, in <module>\\n\")"
            );

            assert!(fields.next().is_none());
        });
    }

    #[test]
    fn err_display() {
        Python::attach(|py| {
            let err = py
                .run(c"raise Exception('banana')", None, None)
                .expect_err("raising should have given us an error");
            assert_eq!(err.to_string(), "Exception: banana");
        });
    }

    #[test]
    fn test_pyerr_send_sync() {
        assert!(value_of!(IsSend, PyErr));
        assert!(value_of!(IsSync, PyErr));

        assert!(value_of!(IsSend, PyErrState));
        assert!(value_of!(IsSync, PyErrState));
    }

    #[test]
    fn test_pyerr_matches() {
        Python::attach(|py| {
            let err = PyErr::new::<PyValueError, _>("foo");
            assert!(err.matches(py, PyValueError::type_object(py)).unwrap());

            assert!(err
                .matches(
                    py,
                    (PyValueError::type_object(py), PyTypeError::type_object(py))
                )
                .unwrap());

            assert!(!err.matches(py, PyTypeError::type_object(py)).unwrap());

            // String is not a valid exception class, so we should get a TypeError
            let err: PyErr = PyErr::from_type(crate::types::PyString::type_object(py), "foo");
            assert!(err.matches(py, PyTypeError::type_object(py)).unwrap());
        })
    }

    #[test]
    fn test_pyerr_cause() {
        Python::attach(|py| {
            let err = py
                .run(c"raise Exception('banana')", None, None)
                .expect_err("raising should have given us an error");
            assert!(err.cause(py).is_none());

            let err = py
                .run(
                    c"raise Exception('banana') from Exception('apple')",
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

    #[test]
    fn warnings() {
        use crate::types::any::PyAnyMethods;
        // Note: although the warning filter is interpreter global, keeping the
        // GIL locked should prevent effects to be visible to other testing
        // threads.
        Python::attach(|py| {
            let cls = py.get_type::<exceptions::PyUserWarning>();

            // Reset warning filter to default state
            let warnings = py.import("warnings").unwrap();
            warnings.call_method0("resetwarnings").unwrap();

            // First, test the warning is emitted
            assert_warnings!(
                py,
                { PyErr::warn(py, &cls, c"I am warning you", 0).unwrap() },
                [(exceptions::PyUserWarning, "I am warning you")]
            );

            // Test with raising
            warnings
                .call_method1("simplefilter", ("error", &cls))
                .unwrap();
            PyErr::warn(py, &cls, c"I am warning you", 0).unwrap_err();

            // Test with error for an explicit module
            warnings.call_method0("resetwarnings").unwrap();
            warnings
                .call_method1("filterwarnings", ("error", "", &cls, "pyo3test"))
                .unwrap();

            // This has the wrong module and will not raise, just be emitted
            assert_warnings!(
                py,
                { PyErr::warn(py, &cls, c"I am warning you", 0).unwrap() },
                [(exceptions::PyUserWarning, "I am warning you")]
            );

            let err = PyErr::warn_explicit(
                py,
                &cls,
                c"I am warning you",
                c"pyo3test.py",
                427,
                None,
                None,
            )
            .unwrap_err();
            assert!(err
                .value(py)
                .getattr("args")
                .unwrap()
                .get_item(0)
                .unwrap()
                .eq("I am warning you")
                .unwrap());

            // Finally, reset filter again
            warnings.call_method0("resetwarnings").unwrap();
        });
    }

    #[test]
    #[cfg(Py_3_11)]
    fn test_add_note() {
        use crate::types::any::PyAnyMethods;
        Python::attach(|py| {
            let err = PyErr::new::<exceptions::PyValueError, _>("original error");
            err.add_note(py, "additional context").unwrap();

            let notes = err.value(py).getattr("__notes__").unwrap();
            assert_eq!(notes.len().unwrap(), 1);
            assert_eq!(
                notes.get_item(0).unwrap().extract::<String>().unwrap(),
                "additional context"
            );
        });
    }
}
