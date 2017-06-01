use std;
use std::ffi::CString;
use std::os::raw::c_char;
use libc;

use ffi;
use python::{ToPythonPointer, IntoPythonPointer, Python, Park, PyDowncastInto, PyClone};
use objects::{PyObject, PyObjectPtr, PyType, PyTypePtr, exc};
use typeob::{PyTypeObject};
use conversion::{ToPyObject, ToPyTuple, IntoPyObject};

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
    py.run("assert CustomError('oops').args == ('oops',)", None, Some(&ctx)).unwrap();
}
```
*/
#[macro_export]
macro_rules! py_exception {
    ($module: ident, $name: ident, $base: ty) => {
        pub struct $name;

        // pyobject_nativetype!($name);

        impl $name {
            pub fn new<'p, T: $crate::ToPyObject>(py: $crate::Python<'p>, args: T) -> $crate::PyErr {
                $crate::PyErr::new::<$name, T>(py, args)
            }
        }

        impl $crate::PyTypeObject for $name {
            #[inline]
            fn type_object<'p>(py: $crate::Python<'p>) -> $crate::PyType<'p> {
                unsafe {
                    #[allow(non_upper_case_globals)]
                    static mut type_object: *mut $crate::ffi::PyTypeObject = 0 as *mut $crate::ffi::PyTypeObject;

                    if type_object.is_null() {
                        type_object = $crate::PyErr::new_type(
                            py,
                            concat!(stringify!($module), ".", stringify!($name)),
                            Some(py.get_type::<$base>()), None).as_type_ptr();
                    }

                    $crate::PyType::from_type_ptr(py, type_object)
                }
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
    pub ptype: PyTypePtr,
    /// The value of the exception.
    ///
    /// This can be either an instance of `ptype`,
    /// a tuple of arguments to be passed to `ptype`'s constructor,
    /// or a single argument to be passed to `ptype`'s constructor.
    /// Call `PyErr::instance()` to get the exception instance in all cases.
    pub pvalue: Option<PyObjectPtr>,
    /// The `PyTraceBack` object associated with the error.
    pub ptraceback: Option<PyObjectPtr>,
}


/// Represents the result of a Python call.
pub type PyResult<T> = Result<T, PyErr>;


// Marker type that indicates an error while downcasting
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
        PyErr::new_helper(py, py.get_type::<T>().park(), value.into_object(py))
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
    pub fn new_type<'p>(py: Python<'p>, name: &str,
                        base: Option<PyType<'p>>, dict: Option<PyObject<'p>>) -> PyType<'p> {
        let base: *mut ffi::PyObject = match base {
            None => std::ptr::null_mut(),
            Some(obj) => obj.into_ptr()
        };

        let dict: *mut ffi::PyObject = match dict {
            None => std::ptr::null_mut(),
            Some(obj) => obj.into_ptr(),
        };

        unsafe {
            let null_terminated_name = CString::new(name).unwrap();
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
                py.get_type::<exc::SystemError>().park()
            } else {
                PyTypePtr::from_owned_ptr(ptype)
            },
            pvalue: PyObjectPtr::from_owned_ptr_or_opt(pvalue),
            ptraceback: PyObjectPtr::from_owned_ptr_or_opt(ptraceback)
        }
    }

    fn new_helper(_py: Python, ty: PyTypePtr, value: PyObjectPtr) -> PyErr {
        assert!(unsafe { ffi::PyExceptionClass_Check(ty.as_ptr()) } != 0);
        PyErr {
            ptype: ty,
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

    fn from_instance_helper<'p>(py: Python, obj: PyObjectPtr) -> PyErr {
        if unsafe { ffi::PyExceptionInstance_Check(obj.as_ptr()) } != 0 {
            PyErr {
                ptype: unsafe { PyTypePtr::from_borrowed_ptr(
                    ffi::PyExceptionInstance_Class(obj.as_ptr())) },
                pvalue: Some(obj),
                ptraceback: None
            }
        } else if unsafe { ffi::PyExceptionClass_Check(obj.as_ptr()) } != 0 {
            PyErr {
                ptype: PyTypePtr::downcast_into(py, obj).unwrap(),
                pvalue: None,
                ptraceback: None
            }
        } else {
            PyErr {
                ptype: py.get_type::<exc::TypeError>().park(),
                pvalue: Some("exceptions must derive from BaseException".into_object(py)),
                ptraceback: None
            }
        }
    }

    /// Construct a new error, with the usual lazy initialization of Python exceptions.
    /// `exc` is the exception type; usually one of the standard exceptions like `py.get_type::<exc::RuntimeError>()`.
    /// `value` is the exception instance, or a tuple of arguments to pass to the exception constructor.
    #[inline]
    pub fn new_lazy_init<'p>(exc: PyType<'p>, value: Option<PyObjectPtr>) -> PyErr {
        PyErr {
            ptype: exc.park(),
            pvalue: value,
            ptraceback: None
        }
    }

    /// Construct a new error, with the usual lazy initialization of Python exceptions.
    /// `exc` is the exception type; usually one of the standard exceptions like `py.get_type::<exc::RuntimeError>()`.
    /// `args` is the a tuple of arguments to pass to the exception constructor.
    #[inline]
    pub fn new_err<'p, A>(py: Python, exc: PyType<'p>, args: A) -> PyErr
        where A: 'p + ToPyTuple
    {
        let pval = args.to_py_tuple(py);
        PyErr {
            ptype: exc.park(),
            pvalue: Some(pval.into_object(py)),
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
    pub fn get_type<'p>(&self, py: Python<'p>) -> PyType<'p> {
        self.ptype.clone_ref(py).cast_into(py).unwrap()
    }

    /// Retrieves the exception instance for this error.
    /// This method takes `&mut self` because the error might need
    /// to be normalized in order to create the exception instance.
    pub fn instance<'p>(&mut self, py: Python<'p>) -> PyObject<'p> {
        self.normalize(py);
        match self.pvalue {
            Some(ref instance) => instance.as_object(py).clone_ref(py),
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
        let message = CString::new(message).unwrap();
        unsafe {
            error_on_minusone(py, ffi::PyErr_WarnEx(
                category.as_ptr(), message.as_ptr(), stacklevel as ffi::Py_ssize_t))
        }
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

/// Convert PyErr to io::Error
impl std::convert::From<PyErr> for std::io::Error {
    fn from(err: PyErr) -> Self {
        std::io::Error::new(
            std::io::ErrorKind::Other, format!("Python exception: {:?}", err))
    }
}

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
