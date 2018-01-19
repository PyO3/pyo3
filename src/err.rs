// Copyright (c) 2017-present PyO3 Project and Contributors
use std;
use std::io;
use std::ffi::CString;
use std::os::raw::c_char;
use std::error::Error;
use libc;

use ffi;
use python::{ToPyPointer, IntoPyPointer, Python};
use object::PyObject;
use objects::{PyObjectRef, PyType, exc};
use instance::Py;
use typeob::PyTypeObject;
use conversion::{ToPyObject, IntoPyObject, ToBorrowedObject};

/// Defines a new exception type.
///
/// # Syntax
/// `py_exception!(module, MyError)`
///
/// * `module` is the name of the containing module.
/// * `MyError` is the name of the new exception type.
///
/// # Example
/// ```
/// #[macro_use]
/// extern crate pyo3;
///
/// use pyo3::{Python, PyDict};
///
/// py_exception!(mymodule, CustomError);
///
/// fn main() {
///     let gil = Python::acquire_gil();
///     let py = gil.python();
///     let ctx = PyDict::new(py);
///
///     ctx.set_item("CustomError", py.get_type::<CustomError>()).unwrap();
///
///     py.run("assert str(CustomError) == \"<class 'mymodule.CustomError'>\"",
///            None, Some(&ctx)).unwrap();
///     py.run("assert CustomError('oops').args == ('oops',)", None, Some(ctx)).unwrap();
/// }
/// ```
#[macro_export]
macro_rules! py_exception {
    ($module: ident, $name: ident, $base: ty) => {
        pub struct $name;

        impl ::std::convert::From<$name> for $crate::PyErr {
            fn from(_err: $name) -> $crate::PyErr {
                $crate::PyErr::new::<$name, _>(())
            }
        }

        impl<T> std::convert::Into<$crate::PyResult<T>> for $name {
            fn into(self) -> $crate::PyResult<T> {
                $crate::PyErr::new::<$name, _>(()).into()
            }
        }

        impl $name {
            pub fn new<T: $crate::ToPyObject + 'static>(args: T) -> $crate::PyErr {
                $crate::PyErr::new::<$name, T>(args)
            }
            pub fn into<R, T: $crate::ToPyObject + 'static>(args: T) -> $crate::PyResult<R> {
                $crate::PyErr::new::<$name, T>(args).into()
            }
            #[inline]
            fn type_object() -> *mut $crate::ffi::PyTypeObject {
                static mut TYPE_OBJECT: *mut $crate::ffi::PyTypeObject =
                    0 as *mut $crate::ffi::PyTypeObject;

                unsafe {
                    if TYPE_OBJECT.is_null() {
                        let gil = $crate::Python::acquire_gil();
                        let py = gil.python();

                        TYPE_OBJECT = $crate::PyErr::new_type(
                            py, concat!(stringify!($module), ".", stringify!($name)),
                            Some(py.get_type::<$base>()), None);
                    }
                    TYPE_OBJECT
                }
            }
        }

        impl $crate::typeob::PyTypeObject for $name {
            #[inline(always)]
            fn init_type() {
                let _ = $name::type_object();
            }

            #[inline]
            fn type_object() -> $crate::Py<$crate::PyType> {
                unsafe {
                    $crate::Py::from_borrowed_ptr(
                        $name::type_object() as *const _ as *mut $crate::ffi::PyObject)
                }
            }
        }
    };
    ($module: ident, $name: ident) => {
        py_exception!($module, $name, $crate::exc::Exception);
    }
}

/// Represents a `PyErr` value
pub enum PyErrValue {
    None,
    Value(PyObject),
    ToArgs(Box<PyErrArguments>),
    ToObject(Box<ToPyObject>),
}

/// Represents a Python exception that was raised.
pub struct PyErr {
    /// The type of the exception. This should be either a `PyClass` or a `PyType`.
    pub ptype: Py<PyType>,

    /// The value of the exception.
    ///
    /// This can be either an instance of `PyObject`,
    /// a tuple of arguments to be passed to `ptype`'s constructor,
    /// or a single argument to be passed to `ptype`'s constructor.
    /// Call `PyErr::instance()` to get the exception instance in all cases.
    pub pvalue: PyErrValue,

    /// The `PyTraceBack` object associated with the error.
    pub ptraceback: Option<PyObject>,
}

/// Represents the result of a Python call.
pub type PyResult<T> = Result<T, PyErr>;

/// Marker type that indicates an error while downcasting
pub struct PyDowncastError;

/// Helper conversion trait that allows to use custom arguments for exception constructor.
pub trait PyErrArguments {
    /// Arguments for exception
    fn arguments(&self, Python) -> PyObject;
}

impl PyErr {
    /// Creates a new PyErr of type `T`.
    ///
    /// `value` can be:
    /// * `NoArgs`: the exception instance will be created using python `T()`
    /// * a tuple: the exception instance will be created using python `T(*tuple)`
    /// * any other value: the exception instance will be created using python `T(value)`
    ///
    /// Panics if `T` is not a python class derived from `BaseException`.
    ///
    /// Example:
    ///  `return Err(PyErr::new::<exc::TypeError, _>("Error message"));`
    pub fn new<T, V>(value: V) -> PyErr
        where T: PyTypeObject, V: ToPyObject + 'static
    {
        let ty = T::type_object();
        assert_ne!(unsafe { ffi::PyExceptionClass_Check(ty.as_ptr()) }, 0);

        PyErr {
            ptype: ty,
            pvalue: PyErrValue::ToObject(Box::new(value)),
            ptraceback: None,
        }
    }

    /// Construct a new error, with the usual lazy initialization of Python exceptions.
    /// `exc` is the exception type; usually one of the standard exceptions
    /// like `exc::RuntimeError`.
    /// `args` is the a tuple of arguments to pass to the exception constructor.
    pub fn from_type<A>(exc: Py<PyType>, args: A) -> PyErr
        where A: ToPyObject + 'static
    {
        PyErr {
            ptype: exc,
            pvalue: PyErrValue::ToObject(Box::new(args)),
            ptraceback: None,
        }
    }

    /// Creates a new PyErr of type `T`.
    pub fn from_value<T>(value: PyErrValue) -> PyErr
        where T: PyTypeObject
    {
        let ty = T::type_object();
        assert_ne!(unsafe { ffi::PyExceptionClass_Check(ty.as_ptr()) }, 0);

        PyErr {
            ptype: ty,
            pvalue: value,
            ptraceback: None,
        }
    }

    /// Creates a new PyErr.
    ///
    /// `obj` must be an Python exception instance, the PyErr will use that instance.
    /// If `obj` is a Python exception type object, the PyErr will (lazily) create a new
    /// instance of that type.
    /// Otherwise, a `TypeError` is created instead.
    pub fn from_instance(obj: &PyObjectRef) -> PyErr {
        let ptr = obj.as_ptr();

        if unsafe { ffi::PyExceptionInstance_Check(ptr) } != 0 {
            PyErr {
                ptype: unsafe { Py::from_borrowed_ptr( ffi::PyExceptionInstance_Class(ptr)) },
                pvalue: PyErrValue::Value(obj.into()),
                ptraceback: None,
            }
        } else if unsafe { ffi::PyExceptionClass_Check(obj.as_ptr()) } != 0 {
            PyErr {
                ptype: unsafe { Py::from_borrowed_ptr(ptr) },
                pvalue: PyErrValue::None,
                ptraceback: None,
            }
        } else {
            PyErr {
                ptype: exc::TypeError::type_object(),
                pvalue: PyErrValue::ToObject(
                    Box::new("exceptions must derive from BaseException")),
                ptraceback: None,
            }
        }
    }

    /// Gets whether an error is present in the Python interpreter's global state.
    #[inline]
    pub fn occurred(_: Python) -> bool {
        unsafe { !ffi::PyErr_Occurred().is_null() }
    }

    /// Retrieves the current error from the Python interpreter's global state.
    /// The error is cleared from the Python interpreter.
    /// If no error is set, returns a `SystemError`.
    pub fn fetch(_: Python) -> PyErr {
        unsafe {
            let mut ptype      : *mut ffi::PyObject = std::ptr::null_mut();
            let mut pvalue     : *mut ffi::PyObject = std::ptr::null_mut();
            let mut ptraceback : *mut ffi::PyObject = std::ptr::null_mut();
            ffi::PyErr_Fetch(&mut ptype, &mut pvalue, &mut ptraceback);
            PyErr::new_from_ffi_tuple(ptype, pvalue, ptraceback)
        }
    }

    /// Creates a new exception type with the given name, which must be of the form
    /// `<module>.<ExceptionName>`, as required by `PyErr_NewException`.
    ///
    /// `base` can be an existing exception type to subclass, or a tuple of classes
    /// `dict` specifies an optional dictionary of class variables and methods
    pub fn new_type<'p>(_: Python<'p>, name: &str, base: Option<&PyType>, dict: Option<PyObject>)
                        -> *mut ffi::PyTypeObject
    {
        let base: *mut ffi::PyObject = match base {
            None => std::ptr::null_mut(),
            Some(obj) => obj.as_ptr()
        };

        let dict: *mut ffi::PyObject = match dict {
            None => std::ptr::null_mut(),
            Some(obj) => obj.as_ptr(),
        };

        unsafe {
            let null_terminated_name = CString::new(name)
                .expect("Failed to initialize nul terminated exception name");
            ffi::PyErr_NewException(
                null_terminated_name.as_ptr() as *mut c_char, base, dict) as *mut ffi::PyTypeObject
        }
    }

    unsafe fn new_from_ffi_tuple(ptype: *mut ffi::PyObject,
                                 pvalue: *mut ffi::PyObject,
                                 ptraceback: *mut ffi::PyObject) -> PyErr {
        // Note: must not panic to ensure all owned pointers get acquired correctly,
        // and because we mustn't panic in normalize().

        let pvalue = if let Some(obj) =
            PyObject::from_owned_ptr_or_opt(Python::assume_gil_acquired(), pvalue)
        {
            PyErrValue::Value(obj)
        } else {
            PyErrValue::None
        };

        PyErr {
            ptype: if ptype.is_null() {
                <exc::SystemError as PyTypeObject>::type_object()
            } else {
                Py::from_owned_ptr(ptype)
            },
            pvalue: pvalue,
            ptraceback: PyObject::from_owned_ptr_or_opt(Python::assume_gil_acquired(), ptraceback),
        }
    }

    /// Print a standard traceback to sys.stderr.
    pub fn print(self, py: Python) {
        self.restore(py);
        unsafe { ffi::PyErr_PrintEx(0) }
    }

    /// Print a standard traceback to sys.stderr.
    pub fn print_and_set_sys_last_vars(self, py: Python) {
        self.restore(py);
        unsafe { ffi::PyErr_PrintEx(1) }
    }

    /// Return true if the current exception matches the exception in `exc`.
    /// If `exc` is a class object, this also returns `true` when `self` is an instance of a subclass.
    /// If `exc` is a tuple, all exceptions in the tuple (and recursively in subtuples) are searched for a match.
    pub fn matches<T>(&self, py: Python, exc: T) -> bool
        where T: ToBorrowedObject
    {
        exc.with_borrowed_ptr(py, |exc| unsafe {
            ffi::PyErr_GivenExceptionMatches(self.ptype.as_ptr(), exc) != 0
        })
    }

    /// Return true if the current exception is instance of `T`
    pub fn is_instance<T>(&self, _py: Python) -> bool
        where T: PyTypeObject
    {
        unsafe {
            ffi::PyErr_GivenExceptionMatches(
                self.ptype.as_ptr(), T::type_object().as_ptr()) != 0
        }
    }

    /// Normalizes the error. This ensures that the exception value is an instance
    /// of the exception type.
    pub fn normalize(&mut self, py: Python) {
        // The normalization helper function involves temporarily moving out of the &mut self,
        // which requires some unsafe trickery:
        unsafe {
            std::ptr::write(self, std::ptr::read(self).into_normalized(py));
        }
        // This is safe as long as normalized() doesn't unwind due to a panic.
    }

    /// Helper function for normalizing the error by deconstructing and reconstructing the PyErr.
    /// Must not panic for safety in normalize()
    fn into_normalized(self, py: Python) -> PyErr {
        let PyErr { ptype, pvalue, ptraceback } = self;

        let mut pvalue = match pvalue {
            PyErrValue::None => std::ptr::null_mut(),
            PyErrValue::Value(ob) => ob.into_ptr(),
            PyErrValue::ToArgs(ob) => ob.arguments(py).into_ptr(),
            PyErrValue::ToObject(ob) => ob.to_object(py).into_ptr(),
        };

        let mut ptype = ptype.into_ptr();
        let mut ptraceback = ptraceback.into_ptr();
        unsafe {
            ffi::PyErr_NormalizeException(&mut ptype, &mut pvalue, &mut ptraceback);
            PyErr::new_from_ffi_tuple(ptype, pvalue, ptraceback)
        }
    }

    /// Retrieves the exception instance for this error.
    /// This method takes `mut self` because the error might need
    /// to be normalized in order to create the exception instance.
    fn instance(mut self, py: Python) -> PyObject {
        self.normalize(py);
        match self.pvalue {
            PyErrValue::Value(ref instance) => instance.clone_ref(py),
            _ => py.None(),
        }
    }

    /// Writes the error back to the Python interpreter's global state.
    /// This is the opposite of `PyErr::fetch()`.
    #[inline]
    pub fn restore(self, py: Python) {
        let PyErr { ptype, pvalue, ptraceback } = self;

        let pvalue = match pvalue {
            PyErrValue::None => std::ptr::null_mut(),
            PyErrValue::Value(ob) => ob.into_ptr(),
            PyErrValue::ToArgs(ob) => ob.arguments(py).into_ptr(),
            PyErrValue::ToObject(ob) => ob.to_object(py).into_ptr(),
        };
        unsafe {
            ffi::PyErr_Restore(ptype.into_ptr(), pvalue, ptraceback.into_ptr())
        }
    }

    /// Issue a warning message.
    /// May return a PyErr if warnings-as-errors is enabled.
    pub fn warn(py: Python, category: &PyObjectRef, message: &str, stacklevel: i32) -> PyResult<()> {
        let message = CString::new(message)?;
        unsafe {
            error_on_minusone(py, ffi::PyErr_WarnEx(
                category.as_ptr(), message.as_ptr(), stacklevel as ffi::Py_ssize_t))
        }
    }

    pub fn clone_ref(&self, py: Python) -> PyErr {
        let v = match self.pvalue {
            PyErrValue::None => PyErrValue::None,
            PyErrValue::Value(ref ob) => PyErrValue::Value(ob.clone_ref(py)),
            PyErrValue::ToArgs(ref ob) => PyErrValue::Value(ob.arguments(py)),
            PyErrValue::ToObject(ref ob) => PyErrValue::Value(ob.to_object(py)),
        };

        let t = if let Some(ref val) = self.ptraceback { Some(val.clone_ref(py))} else { None };
        PyErr {
            ptype: self.ptype.clone_ref(py),
            pvalue: v,
            ptraceback: t,
        }
    }
}

impl std::fmt::Debug for PyErr {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        f.write_str(format!("PyErr {{ type: {:?} }}", self.ptype).as_str())
    }
}

impl IntoPyObject for PyErr {

    fn into_object(self, py: Python) -> PyObject {
        self.instance(py)
    }
}

impl ToPyObject for PyErr {

    fn to_object(&self, py: Python) -> PyObject {
        let err = self.clone_ref(py);
        err.instance(py)
    }
}

impl<'a> IntoPyObject for &'a PyErr {

    fn into_object(self, py: Python) -> PyObject {
        let err = self.clone_ref(py);
        err.instance(py)
    }
}

/// Converts `PyDowncastError` to Python `TypeError`.
impl std::convert::From<PyDowncastError> for PyErr {
    fn from(_err: PyDowncastError) -> PyErr {
        exc::TypeError.into()
    }
}

impl <'p> std::fmt::Debug for PyDowncastError {
    fn fmt(&self, f : &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        f.write_str("PyDowncastError")
    }
}

/// Convert `PyErr` to `io::Error`
impl std::convert::From<PyErr> for std::io::Error {
    fn from(err: PyErr) -> Self {
        std::io::Error::new(
            std::io::ErrorKind::Other, format!("Python exception: {:?}", err))
    }
}

/// Convert `PyErr` to `PyResult<T>`
impl<T> std::convert::Into<PyResult<T>> for PyErr {
    fn into(self) -> PyResult<T> {
        Err(self)
    }
}

macro_rules! impl_to_pyerr {
    ($err: ty, $pyexc: ty) => {
        impl PyErrArguments for $err {
            fn arguments(&self, py: Python) -> PyObject {
                self.description().to_object(py)
            }
        }

        impl std::convert::From<$err> for PyErr {
            fn from(err: $err) -> PyErr {
                PyErr::from_value::<$pyexc>(PyErrValue::ToArgs(Box::new(err)))
            }
        }
    }
}

#[cfg(Py_3)]
/// Create `OSError` from `io::Error`
impl std::convert::From<io::Error> for PyErr {
    fn from(err: io::Error) -> PyErr {
        match err.kind() {
            io::ErrorKind::BrokenPipe =>
                PyErr::from_value::<exc::BrokenPipeError>(PyErrValue::ToArgs(Box::new(err))),
            io::ErrorKind::ConnectionRefused =>
                PyErr::from_value::<exc::ConnectionRefusedError>(
                    PyErrValue::ToArgs(Box::new(err))),
            io::ErrorKind::ConnectionAborted =>
                PyErr::from_value::<exc::ConnectionAbortedError>(
                    PyErrValue::ToArgs(Box::new(err))),
            io::ErrorKind::ConnectionReset =>
                PyErr::from_value::<exc::ConnectionResetError>(
                    PyErrValue::ToArgs(Box::new(err))),
            io::ErrorKind::Interrupted =>
                PyErr::from_value::<exc::InterruptedError>(PyErrValue::ToArgs(Box::new(err))),
            io::ErrorKind::NotFound =>
                PyErr::from_value::<exc::FileNotFoundError>(PyErrValue::ToArgs(Box::new(err))),
            io::ErrorKind::WouldBlock =>
                PyErr::from_value::<exc::BlockingIOError>(PyErrValue::ToArgs(Box::new(err))),
            io::ErrorKind::TimedOut =>
                PyErr::from_value::<exc::TimeoutError>(PyErrValue::ToArgs(Box::new(err))),
            _ =>
                PyErr::from_value::<exc::OSError>(PyErrValue::ToArgs(Box::new(err))),
        }
    }
}

#[cfg(not(Py_3))]
/// Create `OSError` from `io::Error`
impl std::convert::From<io::Error> for PyErr {
    fn from(err: io::Error) -> PyErr {
        PyErr::from_value::<exc::OSError>(PyErrValue::ToArgs(Box::new(err)))
    }
}


/// Extract `errno` and `errdesc` from from `io::Error`
impl PyErrArguments for io::Error {
    fn arguments(&self, py: Python) -> PyObject {
        (self.raw_os_error().unwrap_or(0), self.description()).to_object(py)
    }
}

impl<W: 'static + Send + std::fmt::Debug> std::convert::From<std::io::IntoInnerError<W>> for PyErr
{
    fn from(err: std::io::IntoInnerError<W>) -> PyErr {
        PyErr::from_value::<exc::OSError>(PyErrValue::ToArgs(Box::new(err)))
    }
}

impl<W: Send + std::fmt::Debug> PyErrArguments for std::io::IntoInnerError<W> {
    fn arguments(&self, py: Python) -> PyObject {
        self.description().to_object(py)
    }
}

impl_to_pyerr!(std::num::ParseIntError, exc::ValueError);
impl_to_pyerr!(std::num::ParseFloatError, exc::ValueError);
impl_to_pyerr!(std::string::ParseError, exc::ValueError);
impl_to_pyerr!(std::str::ParseBoolError, exc::ValueError);
impl_to_pyerr!(std::ffi::IntoStringError, exc::UnicodeDecodeError);
impl_to_pyerr!(std::ffi::NulError, exc::ValueError);
impl_to_pyerr!(std::str::Utf8Error, exc::UnicodeDecodeError);
impl_to_pyerr!(std::string::FromUtf8Error, exc::UnicodeDecodeError);
impl_to_pyerr!(std::string::FromUtf16Error, exc::UnicodeDecodeError);
impl_to_pyerr!(std::char::DecodeUtf16Error, exc::UnicodeDecodeError);
impl_to_pyerr!(std::net::AddrParseError, exc::ValueError);

pub fn panic_after_error() -> ! {
    unsafe { ffi::PyErr_Print(); }
    panic!("Python API called failed");
}

/// Returns Ok if the error code is not -1.
#[inline]
pub fn error_on_minusone(py: Python, result: libc::c_int) -> PyResult<()> {
    if result != -1 {
        Ok(())
    } else {
        Err(PyErr::fetch(py))
    }
}

#[cfg(test)]
mod tests {
    use ::{Python, PyErr};
    use objects::exc;

    #[test]
    fn set_typeerror() {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let err: PyErr = exc::TypeError.into();
        err.restore(py);
        assert!(PyErr::occurred(py));
        drop(PyErr::fetch(py));
    }
}
