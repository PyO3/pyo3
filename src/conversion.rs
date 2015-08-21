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
    type ObjectType : PythonObject<'p>;

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
///
/// Usage:
/// ```let obj: PyObject = ...;
/// let prepared = <TargetType as ExtractPyObject>::prepare_extract(&obj);
/// let extracted = try!(extract(&prepared));```
/// 
/// Note: depending on the implementation, the lifetime of the extracted result may
/// depend on the lifetime of the `obj` or the `prepared` variable.
///
/// For example, when extracting `&str` from a python byte string, the resulting string slice will
/// point to the existing string data (lifetime: `'source`).
/// On the other hand, when extracting `&str` from a python unicode string, the preparation step
/// will convert the string to UTF-8, and the resulting string slice will have lifetime `'prepared`.
/// Since only which of these cases applies depends on the runtime type of the python object,
/// both the `obj` and `prepared` variables must outlive the resulting string slice.
///
/// In cases where the result does not depend on the `'prepared` lifetime,
/// the inherent method `PyObject::extract()` can be used.
pub trait ExtractPyObject<'python, 'source, 'prepared> : Sized {
    type Prepared : 'source;

    fn prepare_extract(obj: &'source PyObject<'python>) -> PyResult<'python, Self::Prepared>;

    fn extract(prepared: &'prepared Self::Prepared) -> PyResult<'python, Self>;
}

impl <'python, 'source, 'prepared, T> ExtractPyObject<'python, 'source, 'prepared> for T
where T: PythonObjectWithCheckedDowncast<'python>,
      'python: 'source
{

    type Prepared = &'source PyObject<'python>;

    #[inline]
    fn prepare_extract(obj: &'source PyObject<'python>) -> PyResult<'python, Self::Prepared> {
        Ok(obj)
    }

    #[inline]
    fn extract(&obj: &'prepared &'source PyObject<'python>) -> PyResult<'python, T> {
        Ok(try!(obj.clone().cast_into()))
    }
}

// ToPyObject for references
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


