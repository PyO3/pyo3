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

//! This module contains support for old-style classes. Only available in Python 2.x.

use ffi;
use python::{Python, PythonObject, ToPythonPointer};
use conversion::ToPyObject;
use err::{self, PyResult};
use super::object::PyObject;
use super::tuple::PyTuple;
use super::dict::PyDict;

/// Represents an old-style Python class.
///
/// Only available with Python 2.x.
pub struct PyClass(PyObject);
pyobject_newtype!(PyClass, PyClass_Check, PyClass_Type);

/// Represents an old-style Python instance.
///
/// Only available with Python 2.x.
pub struct PyInstance(PyObject);
pyobject_newtype!(PyInstance, PyInstance_Check, PyInstance_Type);

impl PyClass {
    /// Return true if self is a subclass of base.
    pub fn is_subclass_of(&self, _py: Python, base: &PyClass) -> bool {
        unsafe { ffi::PyClass_IsSubclass(self.as_ptr(), base.as_ptr()) != 0 }
    }

    /// Create a new instance of the class.
    /// The parameters args and kw are used as the positional and keyword parameters to the object’s constructor.
    pub fn create_instance<T>(&self, py: Python, args: T, kw: Option<&PyDict>) -> PyResult<PyInstance>
        where T: ToPyObject<ObjectType=PyTuple>
    {
        args.with_borrowed_ptr(py, |args| unsafe {
            err::result_cast_from_owned_ptr(py,
                ffi::PyInstance_New(self.as_ptr(), args, kw.as_ptr()))
        })
    }

    /// Create a new instance of a specific class without calling its constructor.
    /// The dict parameter will be used as the object’s __dict__.
    pub fn create_instance_raw(&self, py: Python, dict: &PyDict) -> PyResult<PyInstance> {
        unsafe {
            err::result_cast_from_owned_ptr(py,
                ffi::PyInstance_NewRaw(self.as_ptr(), dict.as_object().as_ptr()))
        }
    }
}

