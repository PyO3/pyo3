// Copyright (c) 2016 Daniel Grunwald
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

use libc;
use ffi;
use std::mem;
use python::{Python, PythonObject, PyDrop, ToPythonPointer};
use objects::PyObject;
use function::AbortOnDrop;

// TODO: what's the semantics of the traverse return code?
// If it's just a normal python exception, we might want to use PyErr instead.
pub struct TraverseError(libc::c_int);

#[derive(Copy, Clone)]
pub struct VisitProc<'a> {
    visit: ffi::visitproc,
    arg: *mut libc::c_void,
    /// VisitProc contains a Python instance to ensure that
    /// 1) it is cannot be moved out of the traverse() call
    /// 2) it cannot be sent to other threads
    _py: Python<'a>
}

impl <'a> VisitProc<'a> {
    pub fn call<T>(&self, obj: &T) -> Result<(), TraverseError>
        where T: PythonObject
    {
        let r = unsafe { (self.visit)(obj.as_ptr(), self.arg) };
        if r == 0 {
            Ok(())
        } else {
            Err(TraverseError(r))
        }
    }
}

#[macro_export]
#[doc(hidden)]
macro_rules! py_class_tp_traverse {
    ($class_name:ident,
    /* gc: */ {
        /* traverse_proc: */ None,
        /* traverse_data: */ [ ]
    }) => {
        // If there's nothing to traverse, we don't need to generate
        // tp_traverse.
        // Note that in this case, py_class_type_object_flags! must not
        // use Py_TPFLAGS_HAVE_GC.
        None
    };
    ($class_name:ident,
    /* gc: */ {
        $traverse_proc: expr,
        /* traverse_data: */ []
    }) => {{
        unsafe extern "C" fn tp_traverse(
            slf: *mut $crate::_detail::ffi::PyObject,
            visit: $crate::_detail::ffi::visitproc,
            arg: *mut $crate::_detail::libc::c_void
        ) -> $crate::_detail::libc::c_int
        {
            $crate::py_class::gc::tp_traverse::<$class_name, _>(
                concat!(stringify!($class_name), ".__traverse__"),
                slf, visit, arg, $traverse_proc)
        }
        Some(tp_traverse)
    }};
}

#[doc(hidden)]
pub unsafe fn tp_traverse<C, F>(
    location: &str,
    slf: *mut ffi::PyObject,
    visit: ffi::visitproc,
    arg: *mut libc::c_void,
    callback: F
) -> libc::c_int
where C: PythonObject,
      F: FnOnce(&C, Python, VisitProc) -> Result<(), TraverseError>
{
    let guard = AbortOnDrop(location);
    let py = Python::assume_gil_acquired();
    let visit = VisitProc { visit: visit, arg: arg, _py: py };
    let slf = PyObject::from_borrowed_ptr(py, slf).unchecked_cast_into::<C>();
    let ret = match callback(&slf, py, visit) {
        Ok(()) => 0,
        Err(TraverseError(code)) => code
    };
    slf.release_ref(py);
    mem::forget(guard);
    ret
}

#[macro_export]
#[doc(hidden)]
macro_rules! py_class_tp_clear {
    ($class_name:ident) => {{
        unsafe extern "C" fn tp_clear(
            slf: *mut $crate::_detail::ffi::PyObject
        ) -> $crate::_detail::libc::c_int
        {
            $crate::py_class::gc::tp_clear::<$class_name, _>(
                concat!(stringify!($class_name), ".__clear__"),
                slf, $class_name::__clear__)
        }
        Some(tp_clear)
    }}
}

#[doc(hidden)]
pub unsafe fn tp_clear<C, F>(
    location: &str,
    slf: *mut ffi::PyObject,
    callback: F
) -> libc::c_int
where C: PythonObject,
      F: FnOnce(&C, Python)
{
    let guard = AbortOnDrop(location);
    let py = Python::assume_gil_acquired();
    let slf = PyObject::from_borrowed_ptr(py, slf).unchecked_cast_into::<C>();
    callback(&slf, py);
    slf.release_ref(py);
    mem::forget(guard);
    0
}

/*
/// Trait that has to be implemented by `#[gc_traverse]` members.
pub trait Traversable {
    /// Call VisitProc for all python objects owned by this value.
    fn traverse(&self, py: Python, visit: VisitProc) -> Result<(), TraverseError>;
}

impl <T> Traversable for T where T: PythonObject {
    fn traverse(&self, _py: Python, visit: VisitProc) -> Result<(), TraverseError> {
        visit.call(self.as_object())
    }
}

impl <T> Traversable for Option<T> where T: Traversable {
    fn traverse(&self, py: Python, visit: VisitProc) -> Result<(), TraverseError> {
        match *self {
            Some(ref val) => val.traverse(py, visit),
            None => Ok(())
        }
    }
}

impl <T> Traversable for Vec<T> where T: Traversable {
    fn traverse(&self, py: Python, visit: VisitProc) -> Result<(), TraverseError> {
        for val in self {
            try!(val.traverse(py, visit));
        }
        Ok(())
    }
}
*/

