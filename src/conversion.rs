use std;
use ffi;
use python::{Python, PythonObject, PythonObjectWithCheckedDowncast, ToPythonPointer};
use objects::{exc, PyObject, PyBool, PyTuple};
use err::{self, PyErr, PyResult};

/// Conversion trait that allows various objects to be converted into python objects.
pub trait ToPyObject<'p> {
    type ObjectType : PythonObject<'p> = PyObject<'p>;

    /// Converts self into a python object.
    fn to_py_object(&self, py: Python<'p>) -> Self::ObjectType;

    /// Converts self into a python object.
    ///
    /// May be more efficient than `to_py_object` in some cases because
    /// it can move out of the input object.
    #[inline]
    fn into_py_object(self, py: Python<'p>) -> Self::ObjectType
      where Self: Sized {
        self.to_py_object(py)
    }

    /// Converts self into a python object and calls the specified closure
    /// on the native FFI pointer underlying the python object.
    ///
    /// May be more efficient than `to_py_object` because it does not need
    /// to touch any reference counts when the input object already is a python object.
    #[inline]
    fn with_borrowed_ptr<F, R>(&self, py: Python<'p>, f: F) -> R
      where F: FnOnce(*mut ffi::PyObject) -> R {
        let obj = self.to_py_object(py);
        f(ToPythonPointer::as_ptr(&obj))
    }

    // FFI functions that accept a borrowed reference will use:
    //   input.with_borrowed_ptr(|obj| ffi::Call(obj)
    // 1) input is &PyObject
    //   -> with_borrowed_ptr() just forwards to the closure
    // 2) input is PyObject
    //   -> with_borrowed_ptr() just forwards to the closure
    // 3) input is &str, int, ...
    //   -> to_py_object() allocates new python object; FFI call happens; PyObject::drop() calls Py_DECREF()
    
    // FFI functions that steal a reference will use:
    //   let input = try!(input.into_py_object()); ffi::Call(input.steal_ptr())
    // 1) input is &PyObject
    //   -> into_py_object() calls Py_INCREF
    // 2) input is PyObject
    //   -> into_py_object() is no-op
    // 3) input is &str, int, ...
    //   -> into_py_object() allocates new python object
}

/// FromPyObject is implemented by various types that can be extracted from a python object.
pub trait FromPyObject<'p, 's> {
    fn from_py_object(s: &'s PyObject<'p>) -> PyResult<'p, Self>;
}

// PyObject, PyModule etc.
// We support FromPyObject and ToPyObject for owned python references.
// This allows using existing python objects in code that generically expects a value
// convertible to a python object.

/// Identity conversion: allows using existing `PyObject` instances where
/// `ToPyObject` is expected.
impl <'p, 's> ToPyObject<'p> for PyObject<'s> {
    type ObjectType = PyObject<'p>;

    #[inline]
    fn to_py_object(&self, py: Python<'p>) -> PyObject<'p> {
        self.clone().into_py_object(py)
    }

    #[inline]
    fn into_py_object(self, py: Python<'p>) -> PyObject<'p> {
        // Transmute the lifetime.
        // This is safe, because both lifetime variables represent the same lifetime:
        // that of the python GIL acquisition.
        unsafe { std::mem::transmute(self) }
    }

    #[inline]
    fn with_borrowed_ptr<F, R>(&self, py: Python<'p>, f: F) -> R
      where F: FnOnce(*mut ffi::PyObject) -> R {
        f(self.as_ptr())
    }
}

impl <'p, 's, T> FromPyObject<'p, 's> for T where T: PythonObjectWithCheckedDowncast<'p> {
    #[inline]
    fn from_py_object(s : &'s PyObject<'p>) -> PyResult<'p, T> {
        Ok(try!(s.clone().cast_into()))
    }
}

// &PyObject, &PyModule etc.
// We support FromPyObject and ToPyObject for borrowed python references.
// This allows using existing python objects in code that generically expects a value
// convertible to a python object.
impl <'p, 's, T> ToPyObject<'p> for &'s T where T : ToPyObject<'p> {
    type ObjectType = T::ObjectType;

    #[inline]
    fn to_py_object(&self, py: Python<'p>) -> T::ObjectType {
        (**self).to_py_object(py)
    }

    #[inline]
    fn into_py_object(self, py: Python<'p>) -> T::ObjectType {
        (*self).to_py_object(py)
    }

    #[inline]
    fn with_borrowed_ptr<F, R>(&self, py: Python<'p>, f: F) -> R
      where F: FnOnce(*mut ffi::PyObject) -> R {
        (**self).with_borrowed_ptr(py, f)
    }
}


