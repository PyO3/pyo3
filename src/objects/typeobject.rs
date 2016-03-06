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

use python::{Python, PythonObject, ToPythonPointer};
use conversion::ToPyObject;
use objects::{PyObject, PyTuple, PyDict};
use err::{PyResult, result_from_owned_ptr};
use ffi;
use std::ffi::CStr;
use std::borrow::Cow;

/// Represents a reference to a Python type object.
pub struct PyType(PyObject);

pyobject_newtype!(PyType, PyType_Check, PyType_Type);

impl PyType {
    /// Retrieves the underlying FFI pointer associated with this Python object.
    #[inline]
    pub fn as_type_ptr(&self) -> *mut ffi::PyTypeObject {
        self.0.as_ptr() as *mut ffi::PyTypeObject
    }

    /// Retrieves the PyType instance for the given FFI pointer.
    /// This increments the reference count on the type object.
    /// Undefined behavior if the pointer is NULL or invalid.
    #[inline]
    pub unsafe fn from_type_ptr(py: Python, p: *mut ffi::PyTypeObject) -> PyType {
        PyObject::from_borrowed_ptr(py, p as *mut ffi::PyObject).unchecked_cast_into::<PyType>()
    }

    /// Gets the name of the PyType.
    pub fn name<'a>(&'a self, _py: Python<'a>) -> Cow<'a, str> {
        unsafe {
            CStr::from_ptr((*self.as_type_ptr()).tp_name).to_string_lossy()
        }
    }

    /// Return true if `self` is a subtype of `b`.
    #[inline]
    pub fn is_subtype_of(&self, _: Python, b : &PyType) -> bool {
        unsafe { ffi::PyType_IsSubtype(self.as_type_ptr(), b.as_type_ptr()) != 0 }
    }

    /// Return true if `obj` is an instance of `self`.
    #[inline]
    pub fn is_instance(&self, _: Python, obj : &PyObject) -> bool {
        unsafe { ffi::PyObject_TypeCheck(obj.as_ptr(), self.as_type_ptr()) != 0 }
    }

    /// Calls the type object, thus creating a new instance.
    /// This is equivalent to the Python expression: `self(*args, **kwargs)`
    #[inline]
    pub fn call<A>(&self, py: Python, args: A, kwargs: Option<&PyDict>) -> PyResult<PyObject>
        where A: ToPyObject<ObjectType=PyTuple>
    {
        args.with_borrowed_ptr(py, |args| unsafe {
            result_from_owned_ptr(py, ffi::PyObject_Call(self.0.as_ptr(), args, kwargs.as_ptr()))
        })
    }
}

impl PartialEq for PyType {
    #[inline]
    fn eq(&self, o : &PyType) -> bool {
        self.as_type_ptr() == o.as_type_ptr()
    }
}
impl Eq for PyType { }

