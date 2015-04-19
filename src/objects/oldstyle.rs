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
use err::{self, PyResult};
use super::object::PyObject;
use super::tuple::PyTuple;
use super::dict::PyDict;

pyobject_newtype!(PyClass, PyClass_Check, PyClass_Type);
pyobject_newtype!(PyInstance, PyInstance_Check, PyInstance_Type);

impl <'p> PyClass<'p> {
    /// Return true if self is a subclass of base.
    pub fn is_subclass_of(&self, base: &PyClass<'p>) -> bool {
        unsafe { ffi::PyClass_IsSubclass(self.as_ptr(), base.as_ptr()) != 0 }
    }

    /// Create a new instance of the class.
    /// The parameters arg and kw are used as the positional and keyword parameters to the object’s constructor.
    pub fn create_instance(&self, arg: &PyTuple<'p>, kw: &PyDict<'p>) -> PyResult<'p, PyObject<'p>> {
        unsafe {
            err::result_from_owned_ptr(self.python(),
                ffi::PyInstance_New(self.as_ptr(), arg.as_ptr(), kw.as_ptr()))
        }
    }

    /// Create a new instance of a specific class without calling its constructor.
    /// The dict parameter will be used as the object’s __dict__; if None, a new dictionary will be created for the instance.
    pub fn create_instance_raw(&self, arg: &PyTuple<'p>, kw: Option<&PyDict<'p>>) -> PyResult<'p, PyObject<'p>> {
        unsafe {
            err::result_from_owned_ptr(self.python(),
                ffi::PyInstance_New(self.as_ptr(), arg.as_ptr(), kw.as_ptr()))
        }
    }
}

