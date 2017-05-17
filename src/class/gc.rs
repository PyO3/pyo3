// Copyright (c) 2017-present PyO3 Project and Contributors

//! Python GC support
//!

use std::mem;
use std::os::raw::{c_int, c_void};

use ffi;
use python::{Python, PythonObject, PyDrop, ToPythonPointer};
use objects::PyObject;
use callback::AbortOnDrop;
use class::NO_METHODS;

pub struct PyTraverseError(c_int);

/// GC support
pub trait PyGCProtocol {

    fn __traverse__(&self, py: Python, visit: PyVisit) -> Result<(), PyTraverseError>;

    fn __clear__(&self, py: Python);

}

impl<T> PyGCProtocol for T {
    default fn __traverse__(&self, _: Python, _: PyVisit) -> Result<(), PyTraverseError> {
        Ok(())
    }

    default fn __clear__(&self, _: Python) {}
}

#[doc(hidden)]
pub trait PyGCProtocolImpl {
    fn methods() -> &'static [&'static str];

    fn update_type_object(type_object: &mut ffi::PyTypeObject);
}

impl<T> PyGCProtocolImpl for T where T: PyGCProtocol + PythonObject {
    default fn methods() -> &'static [&'static str] {
        NO_METHODS
    }

    fn update_type_object(type_object: &mut ffi::PyTypeObject) {
        if <T as PyGCProtocolImpl>::methods().is_empty() {
            type_object.tp_flags = ffi::Py_TPFLAGS_DEFAULT
        } else {
            type_object.tp_flags = ffi::Py_TPFLAGS_DEFAULT | ffi::Py_TPFLAGS_HAVE_GC;
            type_object.tp_traverse = Some(tp_traverse::<T>);
            type_object.tp_clear = Some(tp_clear::<T>);
        }
    }
}


#[derive(Copy, Clone)]
pub struct PyVisit<'a> {
    visit: ffi::visitproc,
    arg: *mut c_void,
    /// VisitProc contains a Python instance to ensure that
    /// 1) it is cannot be moved out of the traverse() call
    /// 2) it cannot be sent to other threads
    _py: Python<'a>
}

impl <'a> PyVisit<'a> {
    pub fn call<T>(&self, obj: &T) -> Result<(), PyTraverseError>
        where T: PythonObject
    {
        let r = unsafe { (self.visit)(obj.as_ptr(), self.arg) };
        if r == 0 {
            Ok(())
        } else {
            Err(PyTraverseError(r))
        }
    }
}

#[doc(hidden)]
unsafe extern "C" fn tp_traverse<T>(slf: *mut ffi::PyObject,
                                    visit: ffi::visitproc,
                                    arg: *mut c_void) -> c_int
    where T: PyGCProtocol + PythonObject
{
    const LOCATION: &'static str = concat!(stringify!(T), ".__traverse__()");

    let guard = AbortOnDrop(LOCATION);
    let py = Python::assume_gil_acquired();
    let visit = PyVisit { visit: visit, arg: arg, _py: py };
    let slf = PyObject::from_borrowed_ptr(py, slf).unchecked_cast_into::<T>();

    let ret = match T::__traverse__(&slf, py, visit) {
        Ok(()) => 0,
        Err(PyTraverseError(code)) => code
    };
    slf.release_ref(py);
    mem::forget(guard);
    ret
}

unsafe extern "C" fn tp_clear<T>(slf: *mut ffi::PyObject) -> c_int
    where T: PyGCProtocol + PythonObject
{
    const LOCATION: &'static str = concat!(stringify!(T), ".__clear__()");

    let guard = AbortOnDrop(LOCATION);
    let py = Python::assume_gil_acquired();
    let slf = PyObject::from_borrowed_ptr(py, slf).unchecked_cast_into::<T>();
    T::__clear__(&slf, py);
    slf.release_ref(py);
    mem::forget(guard);
    0
}
