// Copyright (c) 2015 Daniel Grunwald
//
// Permission is hereby granted, free of charge, to any person obtaining a copy of this
// software and associated documentation files (the "Software"), to deal in the Software
// without restriction, including without limitation the rights to use, copy, modify, merge,
// publish, distribute, sublicense, and/or sell copies of the Software, and to permit persons
// to whom the Software is furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in all copies or
// substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED,
// INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR
// PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE
// FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR
// OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER
// DEALINGS IN THE SOFTWARE.

use std;
use ffi;
use python::{Python, PythonObject, PythonObjectWithCheckedDowncast, ToPythonPointer};
use objects::PyObject;
use err::PyResult;

/// Conversion trait that allows various objects to be converted into Python objects.
pub trait ToPyObject<'p> {
    type ObjectType : PythonObject<'p> = PyObject<'p>;

    /// Converts self into a Python object.
    fn to_py_object(&self, py: Python<'p>) -> Self::ObjectType;

    /// Converts self into a Python object.
    ///
    /// May be more efficient than `to_py_object` in some cases because
    /// it can move out of the input object.
    #[inline]
    fn into_py_object(self, py: Python<'p>) -> Self::ObjectType
      where Self: Sized {
        self.to_py_object(py)
    }

    /// Converts self into a Python object and calls the specified closure
    /// on the native FFI pointer underlying the Python object.
    ///
    /// May be more efficient than `to_py_object` because it does not need
    /// to touch any reference counts when the input object already is a Python object.
    #[inline]
    fn with_borrowed_ptr<F, R>(&self, py: Python<'p>, f: F) -> R
      where F: FnOnce(*mut ffi::PyObject) -> R {
        let obj = self.to_py_object(py).into_object();
        f(obj.as_ptr())
    }

    // FFI functions that accept a borrowed reference will use:
    //   input.with_borrowed_ptr(|obj| ffi::Call(obj)
    // 1) input is &PyObject
    //   -> with_borrowed_ptr() just forwards to the closure
    // 2) input is PyObject
    //   -> with_borrowed_ptr() just forwards to the closure
    // 3) input is &str, int, ...
    //   -> to_py_object() allocates new Python object; FFI call happens; PyObject::drop() calls Py_DECREF()
    
    // FFI functions that steal a reference will use:
    //   let input = try!(input.into_py_object()); ffi::Call(input.steal_ptr())
    // 1) input is &PyObject
    //   -> into_py_object() calls Py_INCREF
    // 2) input is PyObject
    //   -> into_py_object() is no-op
    // 3) input is &str, int, ...
    //   -> into_py_object() allocates new Python object
}

/// FromPyObject is implemented by various types that can be extracted from a Python object.
pub trait FromPyObject<'p> {
    fn from_py_object(s: &PyObject<'p>) -> PyResult<'p, Self>;
}

// PyObject, PyModule etc.
// We support FromPyObject and ToPyObject for owned python references.
// This allows using existing Python objects in code that generically expects a value
// convertible to a Python object.

/// Identity conversion: allows using existing `PyObject` instances where
/// `ToPyObject` is expected.
impl <'p, 's> ToPyObject<'p> for PyObject<'s> {
    type ObjectType = PyObject<'p>;

    #[inline]
    fn to_py_object(&self, py: Python<'p>) -> PyObject<'p> {
        self.clone().into_py_object(py)
    }

    #[inline]
    fn into_py_object(self, _py: Python<'p>) -> PyObject<'p> {
        // Transmute the lifetime.
        // This is safe, because both lifetime variables represent the same lifetime:
        // that of the python GIL acquisition.
        unsafe { std::mem::transmute(self) }
    }

    #[inline]
    fn with_borrowed_ptr<F, R>(&self, _py: Python<'p>, f: F) -> R
      where F: FnOnce(*mut ffi::PyObject) -> R {
        f(self.as_ptr())
    }
}

impl <'p, T> FromPyObject<'p> for T where T: PythonObjectWithCheckedDowncast<'p> {
    #[inline]
    fn from_py_object(s : &PyObject<'p>) -> PyResult<'p, T> {
        Ok(try!(s.clone().cast_into()))
    }
}

// &PyObject, &PyModule etc.
// We support FromPyObject and ToPyObject for borrowed python references.
// This allows using existing Python objects in code that generically expects a value
// convertible to a Python object.
impl <'p, 's, T: ?Sized> ToPyObject<'p> for &'s T where T: ToPyObject<'p> {
    type ObjectType = T::ObjectType;

    #[inline]
    fn to_py_object(&self, py: Python<'p>) -> T::ObjectType {
        <T as ToPyObject>::to_py_object(*self, py)
    }

    #[inline]
    fn into_py_object(self, py: Python<'p>) -> T::ObjectType {
        <T as ToPyObject>::to_py_object(self, py)
    }

    #[inline]
    fn with_borrowed_ptr<F, R>(&self, py: Python<'p>, f: F) -> R
      where F: FnOnce(*mut ffi::PyObject) -> R {
        <T as ToPyObject>::with_borrowed_ptr(*self, py, f)
    }
}


