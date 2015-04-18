use std;
use ffi;
use python::{Python, PythonObject, PythonObjectWithCheckedDowncast, ToPythonPointer};
use objects::{exc, PyObject, PyBool, PyTuple};
use err::{self, PyErr, PyResult};

/// ToPyObject is implemented for types that can be converted into a python object.
/// The goal is to allow methods that take a python object to take anything that
/// can be converted into a python object.
/// For example, compare calling the following method signatures:
///   fn m1(o: &PyObject) {}
///   fn m2<O>(o: &O) where O : ToPyObject {}
///
///   let o: &PyObject = ...;
///   m1(o);
///   m2(o);
///
///   let p: PyPtr<PyObject> = ...;
///   m1(*p)
///   m2(p)
///
///   let i: i32 = ...;
///   m1(*try!(i.to_py_object(py)))
///   m2(i)
pub trait ToPyObject<'p> {
    type ObjectType : PythonObject<'p> = PyObject<'p>;

    fn to_py_object(&self, py: Python<'p>) -> PyResult<'p, Self::ObjectType>;

    #[inline]
    fn into_py_object(self, py: Python<'p>) -> PyResult<'p, Self::ObjectType>
      where Self: Sized {
        self.to_py_object(py)
    }

    #[inline]
    fn with_borrowed_ptr<F, R>(&self, py: Python<'p>, f: F) -> PyResult<'p, R>
      where F: FnOnce(*mut ffi::PyObject) -> PyResult<'p, R> {
        let obj = try!(self.to_py_object(py));
        f(ToPythonPointer::as_ptr(&obj))
    }

    // FFI functions that accept a borrowed reference will use:
    //   input.with_borrowed_ptr(|obj| ffi::Call(obj)
    // 1) input is &PyObject
    //   -> with_borrowed_ptr() just forwards to the closure
    // 2) input is PyObject
    //   -> with_borrowed_ptr() just forwards to the closure
    // 3) input is &str, int, ...
    //   -> to_py_object() allocates new python object; FFI call happens; PyPtr::drop() calls Py_DECREF()
    
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

impl <'p> ToPyObject<'p> for PyObject<'p> {
    type ObjectType = PyObject<'p>;

    #[inline]
    fn to_py_object(&self, py: Python<'p>) -> PyResult<'p, PyObject<'p>> {
        Ok(self.clone())
    }

    #[inline]
    fn into_py_object(self, py: Python<'p>) -> PyResult<'p, PyObject<'p>> {
        Ok(self)
    }

    #[inline]
    fn with_borrowed_ptr<F, R>(&self, py: Python<'p>, f: F) -> PyResult<'p, R>
      where F: FnOnce(*mut ffi::PyObject) -> PyResult<'p, R> {
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
    type ObjectType = <T as ToPyObject<'p>>::ObjectType;

    #[inline]
    fn to_py_object(&self, py: Python<'p>) -> PyResult<'p, <T as ToPyObject<'p>>::ObjectType> {
        (**self).to_py_object(py)
    }

    #[inline]
    fn into_py_object(self, py: Python<'p>) -> PyResult<'p, <T as ToPyObject<'p>>::ObjectType> {
        (*self).to_py_object(py)
    }

    #[inline]
    fn with_borrowed_ptr<F, R>(&self, py: Python<'p>, f: F) -> PyResult<'p, R>
      where F: FnOnce(*mut ffi::PyObject) -> PyResult<'p, R> {
        (**self).with_borrowed_ptr(py, f)
    }
}


