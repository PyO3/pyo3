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

/// Represents a reference to a Python type object.
pub struct PyType<'p>(PyObject<'p>);

pyobject_newtype!(PyType, PyType_Check, PyType_Type);

impl <'p> PyType<'p> {
    /// Retrieves the underlying FFI pointer associated with this Python object.
    #[inline]
    pub fn as_type_ptr(&self) -> *mut ffi::PyTypeObject {
        self.0.as_ptr() as *mut ffi::PyTypeObject
    }

    /// Retrieves the PyType instance for the given FFI pointer.
    /// Undefined behavior if the pointer is NULL or invalid.
    #[inline]
    pub unsafe fn from_type_ptr<'a>(py: Python<'p>, p: *mut ffi::PyTypeObject) -> PyType<'p> {
        PyObject::from_borrowed_ptr(py, p as *mut ffi::PyObject).unchecked_cast_into::<PyType>()
    }

    /// Return true if self is a subtype of b.
    #[inline]
    pub fn is_subtype_of(&self, b : &PyType<'p>) -> bool {
        unsafe { ffi::PyType_IsSubtype(self.as_type_ptr(), b.as_type_ptr()) != 0 }
    }

    /// Return true if obj is an instance of self.
    #[inline]
    pub fn is_instance(&self, obj : &PyObject<'p>) -> bool {
        unsafe { ffi::PyObject_TypeCheck(obj.as_ptr(), self.as_type_ptr()) != 0 }
    }

    /// Calls the type object, thus creating a new instance.
    /// This is equivalent to the Python expression: `self(*args, **kwargs)`
    #[inline]
    pub fn call<A>(&self, args: A, kwargs: Option<&PyDict<'p>>) -> PyResult<'p, PyObject<'p>>
      where A: ToPyObject<'p, ObjectType=PyTuple<'p>> {
        let py = self.python();
        args.with_borrowed_ptr(py, |args| unsafe {
            result_from_owned_ptr(py, ffi::PyObject_Call(self.0.as_ptr(), args, kwargs.as_ptr()))
        })
    }
}

impl <'p> PartialEq for PyType<'p> {
    #[inline]
    fn eq(&self, o : &PyType<'p>) -> bool {
        self.as_type_ptr() == o.as_type_ptr()
    }
}
impl <'p> Eq for PyType<'p> { }

