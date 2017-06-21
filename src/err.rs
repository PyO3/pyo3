use std;
use std::io;
use std::ffi::CString;
use std::os::raw::c_char;
use std::error::Error;
use libc;

use ffi;
use python::{ToPyPointer, IntoPyPointer, Python, PyDowncastFrom, PyClone};
use objects::{PyObject, PyType, exc};
use token::Py;
use typeob::PyTypeObject;
use conversion::{ToPyObject, IntoPyTuple, IntoPyObject};

/**
Defines a new exception type.

# Syntax
`py_exception!(module, MyError)`

* `module` is the name of the containing module.
* `MyError` is the name of the new exception type.

# Example
```
#[macro_use]
extern crate pyo3;

use pyo3::{Python, PyDict};

py_exception!(mymodule, CustomError);

fn main() {
let gil = Python::acquire_gil();
    let py = gil.python();
    let ctx = PyDict::new(py);

    ctx.set_item("CustomError", py.get_type::<CustomError>()).unwrap();

    py.run("assert str(CustomError) == \"<class 'mymodule.CustomError'>\"", None, Some(&ctx)).unwrap();
    py.run("assert CustomError('oops').args == ('oops',)", None, Some(ctx)).unwrap();
}
```
*/
#[macro_export]
macro_rules! py_exception {
    ($module: ident, $name: ident, $base: ty) => {
        pub struct $name;

        // pyobject_nativetype!($name);

        impl $name {
            pub fn new<T: $crate::IntoPyObject>(py: $crate::Python, args: T) -> $crate::PyErr {
                $crate::PyErr::new::<$name, T>(py, args)
            }
            #[inline(always)]
            fn type_object(py: $crate::Python) -> *mut $crate::ffi::PyTypeObject {
                #[allow(non_upper_case_globals)]
                static mut type_object: *mut $crate::ffi::PyTypeObject =
                    0 as *mut $crate::ffi::PyTypeObject;

                unsafe {
                    if type_object.is_null() {
                        type_object = $crate::PyErr::new_type(
                            py, concat!(stringify!($module), ".", stringify!($name)),
                            Some(py.get_type::<$base>()), None).as_type_ptr();
                    }
                    type_object
                }
            }
        }

        impl $crate::typeob::PyTypeObject for $name {
            #[inline(always)]
            fn init_type(py: $crate::Python) {
                let _ = $name::type_object(py);
            }

            #[inline]
            fn type_object<'p>(py: $crate::Python<'p>) -> &'p $crate::PyType {
                unsafe { $crate::PyType::from_type_ptr(py, $name::type_object(py)) }
            }
        }
    };
    ($module: ident, $name: ident) => {
        py_exception!($module, $name, $crate::exc::Exception);
    }
}

/// Represents a Python exception that was raised.
#[derive(Debug)]
pub struct PyErr {
    /// The type of the exception. This should be either a `PyClass` or a `PyType`.
    pub ptype: Py<PyType>,
    /// The value of the exception.
    ///
    /// This can be either an instance of `ptype`,
    /// a tuple of arguments to be passed to `ptype`'s constructor,
    /// or a single argument to be passed to `ptype`'s constructor.
    /// Call `PyErr::instance()` to get the exception instance in all cases.
    pub pvalue: Option<PyObject>,
    /// The `PyTraceBack` object associated with the error.
    pub ptraceback: Option<PyObject>,
}


/// Represents the result of a Python call.
pub type PyResult<T> = Result<T, PyErr>;


/// Marker type that indicates an error while downcasting
pub struct PyDowncastError<'p>(pub Python<'p>, pub Option<&'p str>);


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
    ///  `return Err(PyErr::new::<exc::TypeError, _>(py, "Error message"));`
    pub fn new<T, V>(py: Python, value: V) -> PyErr
        where T: PyTypeObject, V: IntoPyObject
    {
        PyErr::new_helper(py, py.get_type::<T>(), value.into_object(py))
    }

    /// Gets whether an error is present in the Python interpreter's global state.
    #[inline]
    pub fn occurred(_ : Python) -> bool {
        unsafe { !ffi::PyErr_Occurred().is_null() }
    }

    /// Creates a new exception type with the given name, which must be of the form
    /// `<module>.<ExceptionName>`, as required by `PyErr_NewException`.
    ///
    /// `base` can be an existing exception type to subclass, or a tuple of classes
    /// `dict` specifies an optional dictionary of class variables and methods
    pub fn new_type<'p>(py: Python<'p>,
                        name: &str, base: Option<&PyType>, dict: Option<PyObject>) -> &'p PyType
    {
        let base: *mut ffi::PyObject = match base {
            None => std::ptr::null_mut(),
            Some(obj) => obj.into_ptr()
        };

        let dict: *mut ffi::PyObject = match dict {
            None => std::ptr::null_mut(),
            Some(obj) => obj.into_ptr(),
        };

        unsafe {
            let null_terminated_name = CString::new(name).expect("Failed to initialize nul terminated exception name");
            let ptr = ffi::PyErr_NewException(
                null_terminated_name.as_ptr() as *mut c_char,
                base, dict) as *mut ffi::PyTypeObject;
            PyType::from_type_ptr(py, ptr)
        }
    }

    /// Retrieves the current error from the Python interpreter's global state.
    /// The error is cleared from the Python interpreter.
    /// If no error is set, returns a `SystemError`.
    pub fn fetch(py: Python) -> PyErr {
        unsafe {
            let mut ptype      : *mut ffi::PyObject = std::ptr::null_mut();
            let mut pvalue     : *mut ffi::PyObject = std::ptr::null_mut();
            let mut ptraceback : *mut ffi::PyObject = std::ptr::null_mut();
            ffi::PyErr_Fetch(&mut ptype, &mut pvalue, &mut ptraceback);
            PyErr::new_from_ffi_tuple(py, ptype, pvalue, ptraceback)
        }
    }

    unsafe fn new_from_ffi_tuple(py: Python,
                                 ptype: *mut ffi::PyObject,
                                 pvalue: *mut ffi::PyObject,
                                 ptraceback: *mut ffi::PyObject) -> PyErr {
        // Note: must not panic to ensure all owned pointers get acquired correctly,
        // and because we mustn't panic in normalize().
        PyErr {
            ptype: if ptype.is_null() {
                py.get_type::<exc::SystemError>().into()
            } else {
                PyType::from_type_ptr(py, ptype as *mut ffi::PyTypeObject).into()
            },
            pvalue: PyObject::from_owned_ptr_or_opt(py, pvalue),
            ptraceback: PyObject::from_owned_ptr_or_opt(py, ptraceback)
        }
    }

    fn new_helper(_py: Python, ty: &PyType, value: PyObject) -> PyErr {
        assert!(unsafe { ffi::PyExceptionClass_Check(ty.as_ptr()) } != 0);
        PyErr {
            ptype: ty.into(),
            pvalue: Some(value),
            ptraceback: None
        }
    }

    /// Creates a new PyErr.
    ///
    /// `obj` must be an Python exception instance, the PyErr will use that instance.
    /// If `obj` is a Python exception type object, the PyErr will (lazily) create a new instance of that type.
    /// Otherwise, a `TypeError` is created instead.
    pub fn from_instance<O>(py: Python, obj: O) -> PyErr where O: IntoPyObject {
        PyErr::from_instance_helper(py, obj.into_object(py))
    }

    fn from_instance_helper<'p>(py: Python, obj: PyObject) -> PyErr {
        let ptr = obj.as_ptr();

        if unsafe { ffi::PyExceptionInstance_Check(ptr) } != 0 {
            PyErr {
                ptype: unsafe { PyType::from_type_ptr(py, ptr as *mut ffi::PyTypeObject).into() },
                pvalue: Some(obj),
                ptraceback: None
            }
        } else if unsafe { ffi::PyExceptionClass_Check(obj.as_ptr()) } != 0 {
            PyErr {
                ptype: PyType::downcast_from(py, &obj)
                    .expect("Failed to downcast into PyType").into(),
                pvalue: None,
                ptraceback: None
            }
        } else {
            PyErr {
                ptype: py.get_type::<exc::TypeError>().into(),
                pvalue: Some("exceptions must derive from BaseException".into_object(py)),
                ptraceback: None
            }
        }
    }

    /// Construct a new error, with the usual lazy initialization of Python exceptions.
    /// `exc` is the exception type; usually one of the standard exceptions like `py.get_type::<exc::RuntimeError>()`.
    /// `value` is the exception instance, or a tuple of arguments to pass to the exception constructor.
    #[inline]
    pub fn new_lazy_init(exc: &PyType, value: Option<PyObject>) -> PyErr {
        PyErr {
            ptype: exc.into(),
            pvalue: value,
            ptraceback: None
        }
    }

    /// Construct a new error, with the usual lazy initialization of Python exceptions.
    /// `exc` is the exception type; usually one of the standard exceptions like `py.get_type::<exc::RuntimeError>()`.
    /// `args` is the a tuple of arguments to pass to the exception constructor.
    #[inline]
    pub fn new_err<A>(py: Python, exc: &PyType, args: A) -> PyErr
        where A: IntoPyTuple
    {
        PyErr {
            ptype: exc.into(),
            pvalue: Some(args.into_tuple(py).into()),
            ptraceback: None
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
        where T: ToPyObject
    {
        exc.with_borrowed_ptr(py, |exc| unsafe {
            ffi::PyErr_GivenExceptionMatches(self.ptype.as_ptr(), exc) != 0
        })
    }

    /// Normalizes the error. This ensures that the exception value is an instance of the exception type.
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
        let mut ptype = ptype.into_ptr();
        let mut pvalue = pvalue.into_ptr();
        let mut ptraceback = ptraceback.into_ptr();
        unsafe {
            ffi::PyErr_NormalizeException(&mut ptype, &mut pvalue, &mut ptraceback);
            PyErr::new_from_ffi_tuple(py, ptype, pvalue, ptraceback)
        }
    }

    /// Retrieves the exception type.
    pub fn get_type(&self, py: Python) -> Py<PyType> {
        self.ptype.clone_ref(py)
    }

    /// Retrieves the exception instance for this error.
    /// This method takes `&mut self` because the error might need
    /// to be normalized in order to create the exception instance.
    pub fn instance(&mut self, py: Python) -> PyObject {
        self.normalize(py);
        match self.pvalue {
            Some(ref instance) => instance.to_object(py),
            None => py.None(),
        }
    }

    /// Writes the error back to the Python interpreter's global state.
    /// This is the opposite of `PyErr::fetch()`.
    #[inline]
    pub fn restore(self, _py: Python) {
        let PyErr { ptype, pvalue, ptraceback } = self;
        unsafe {
            ffi::PyErr_Restore(ptype.into_ptr(), pvalue.into_ptr(), ptraceback.into_ptr())
        }
    }

    /// Issue a warning message.
    /// May return a PyErr if warnings-as-errors is enabled.
    pub fn warn(py: Python, category: &PyObject, message: &str, stacklevel: i32) -> PyResult<()> {
        let message = CString::new(message).map_err(|e| e.to_pyerr(py))?;
        unsafe {
            error_on_minusone(py, ffi::PyErr_WarnEx(
                category.as_ptr(), message.as_ptr(), stacklevel as ffi::Py_ssize_t))
        }
    }

    pub fn clone_ref(&self, py: Python) -> PyErr {
        PyErr {
            ptype: self.ptype.clone_ref(py),
            pvalue: self.pvalue.clone_ref(py),
            ptraceback: self.ptraceback.clone_ref(py),
        }
    }

    pub fn release(self, py: Python) {
        let PyErr { ptype, pvalue, ptraceback } = self;
        py.release(ptype);
        py.release(pvalue);
        py.release(ptraceback);
    }
}

/// Converts `PyDowncastError` to Python `TypeError`.
impl <'p> std::convert::From<PyDowncastError<'p>> for PyErr {
    fn from(err: PyDowncastError<'p>) -> PyErr {
        PyErr::new_lazy_init(err.0.get_type::<exc::TypeError>(), None)
    }
}

impl <'p> std::fmt::Debug for PyDowncastError<'p> {
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

/// Converts into `PyErr`
pub trait ToPyErr {
    fn to_pyerr(&self, Python) -> PyErr;
}

macro_rules! impl_to_pyerr {
    ($err: ty, $pyexc: ty) => {
        impl $crate::ToPyErr for $err {
            fn to_pyerr(&self, py: $crate::Python) -> PyErr {
                PyErr::new::<$pyexc, _>(py, self.description())
            }
        }
    }
}

#[cfg(Py_3)]
/// Create `OSError` from `io::Error`
impl ToPyErr for io::Error {

    fn to_pyerr(&self, py: Python) -> PyErr {
        let tp = match self.kind() {
            io::ErrorKind::BrokenPipe => py.get_type::<exc::BrokenPipeError>(),
            io::ErrorKind::ConnectionRefused => py.get_type::<exc::ConnectionRefusedError>(),
            io::ErrorKind::ConnectionAborted => py.get_type::<exc::ConnectionAbortedError>(),
            io::ErrorKind::ConnectionReset => py.get_type::<exc::ConnectionResetError>(),
            io::ErrorKind::Interrupted => py.get_type::<exc::InterruptedError>(),
            io::ErrorKind::NotFound => py.get_type::<exc::FileNotFoundError>(),
            io::ErrorKind::WouldBlock => py.get_type::<exc::BlockingIOError>(),
            io::ErrorKind::TimedOut => py.get_type::<exc::TimeoutError>(),
            _ => py.get_type::<exc::OSError>(),
        };

        let errno = self.raw_os_error().unwrap_or(0);
        let errdesc = self.description();

        PyErr::new_err(py, &tp, (errno, errdesc))
    }
}

#[cfg(not(Py_3))]
/// Create `OSError` from `io::Error`
impl ToPyErr for io::Error {

    fn to_pyerr(&self, py: Python) -> PyErr {
        let errno = self.raw_os_error().unwrap_or(0);
        let errdesc = self.description();

        PyErr::new_err(py, &py.get_type::<exc::OSError>(), (errno, errdesc))
    }
}

impl<W: Send + std::fmt::Debug> ToPyErr for std::io::IntoInnerError<W> {
    fn to_pyerr(&self, py: Python) -> PyErr {
        PyErr::new::<exc::OSError, _>(py, self.description())
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
        PyErr::new_lazy_init(py.get_type::<exc::TypeError>(), None).restore(py);
        assert!(PyErr::occurred(py));
        drop(PyErr::fetch(py));
    }
}
