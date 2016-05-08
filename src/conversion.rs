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
use python::{Python, PythonObject, PythonObjectWithCheckedDowncast, PyClone};
use objects::PyObject;
use err::PyResult;

/// Conversion trait that allows various objects to be converted into Python objects.
pub trait ToPyObject {
    type ObjectType : PythonObject;

    /// Converts self into a Python object.
    fn to_py_object(&self, py: Python) -> Self::ObjectType;

    /// Converts self into a Python object.
    ///
    /// May be more efficient than `to_py_object` in some cases because
    /// it can move out of the input object.
    #[inline]
    fn into_py_object(self, py: Python) -> Self::ObjectType
      where Self: Sized
    {
        self.to_py_object(py)
    }

    /// Converts self into a Python object and calls the specified closure
    /// on the native FFI pointer underlying the Python object.
    ///
    /// May be more efficient than `to_py_object` because it does not need
    /// to touch any reference counts when the input object already is a Python object.
    #[inline]
    fn with_borrowed_ptr<F, R>(&self, py: Python, f: F) -> R
        where F: FnOnce(*mut ffi::PyObject) -> R
    {
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

py_impl_to_py_object_for_python_object!(PyObject);

/// FromPyObject is implemented by various types that can be extracted from a Python object.
///
/// Normal usage is through the `PyObject::extract` helper method:
/// ```let obj: PyObject = ...;
/// let value = try!(obj.extract::<TargetType>(py));
/// ```
///
/// TODO: update this documentation
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
pub trait FromPyObject<'source> : Sized {
    /// Extracts `Self` from the source `PyObject`.
    fn extract(py: Python, obj: &'source PyObject) -> PyResult<Self>;
}


py_impl_from_py_object_for_python_object!(PyObject);



pub trait RefFromPyObject {
    fn with_extracted<F, R>(py: Python, obj: &PyObject, f: F) -> PyResult<R>
        where F: FnOnce(&Self) -> R;
}

impl <T: ?Sized> RefFromPyObject for T
    where for<'a> &'a T: FromPyObject<'a>
{
    #[inline]
    fn with_extracted<F, R>(py: Python, obj: &PyObject, f: F) -> PyResult<R>
        where F: FnOnce(&Self) -> R
    {
        match FromPyObject::extract(py, obj) {
            Ok(val) => Ok(f(val)),
            Err(e) => Err(e)
        }
    }
}

/*
impl <'prepared, T> ExtractPyObject<'prepared> for T
where T: PythonObjectWithCheckedDowncast
{
    type Prepared = PyObject;

    #[inline]
    fn prepare_extract(py: Python, obj: &PyObject) -> PyResult<Self::Prepared> {
        Ok(obj.clone_ref(py))
    }

    #[inline]
    fn extract(py: Python, obj: &'prepared Self::Prepared) -> PyResult<T> {
        Ok(try!(obj.clone_ref(py).cast_into(py)))
    }
}
*/

// ToPyObject for references
impl <'a, T: ?Sized> ToPyObject for &'a T where T: ToPyObject {
    type ObjectType = T::ObjectType;

    #[inline]
    fn to_py_object(&self, py: Python) -> T::ObjectType {
        <T as ToPyObject>::to_py_object(*self, py)
    }

    #[inline]
    fn into_py_object(self, py: Python) -> T::ObjectType {
        <T as ToPyObject>::to_py_object(self, py)
    }

    #[inline]
    fn with_borrowed_ptr<F, R>(&self, py: Python, f: F) -> R
        where F: FnOnce(*mut ffi::PyObject) -> R
    {
        <T as ToPyObject>::with_borrowed_ptr(*self, py, f)
    }
}

/// `Option::Some<T>` is converted like `T`.
/// `Option::None` is converted to Python `None`.
impl <T> ToPyObject for Option<T> where T: ToPyObject {
    type ObjectType = PyObject;

    fn to_py_object(&self, py: Python) -> PyObject {
        match *self {
            Some(ref val) => val.to_py_object(py).into_object(),
            None => py.None()
        }
    }

    fn into_py_object(self, py: Python) -> PyObject {
        match self {
            Some(val) => val.into_py_object(py).into_object(),
            None => py.None()
        }
    }
}

impl <'source, T> FromPyObject<'source> for Option<T> where T: FromPyObject<'source> {
    fn extract(py: Python, obj: &'source PyObject) -> PyResult<Self> {
        if obj.as_ptr() == unsafe { ffi::Py_None() } {
            Ok(None)
        } else {
            match T::extract(py, obj) {
                Ok(v) => Ok(Some(v)),
                Err(e) => Err(e)
            }
        }
    }
}

/*
impl <'prepared, T> ExtractPyObject<'prepared> for Option<T>
where T: ExtractPyObject<'prepared>
{
    type Prepared = Option<T::Prepared>;

    fn prepare_extract(py: Python, obj: &PyObject) -> PyResult<Self::Prepared> {
        if obj.as_ptr() == unsafe { ffi::Py_None() } {
            Ok(None)
        } else {
            Ok(Some(try!(T::prepare_extract(py, obj))))
        }
    }

    fn extract(py: Python, obj: &'prepared Self::Prepared) -> PyResult<Option<T>> {
        match *obj {
            Some(ref inner) => {
                match T::extract(py, inner) {
                    Ok(v) => Ok(Some(v)),
                    Err(e) => Err(e)
                }
            },
            None => Ok(None)
        }
    }
}
*/

