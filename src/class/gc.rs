// Copyright (c) 2017-present PyO3 Project and Contributors

//! Python GC support
//!

use std::mem;
use std::os::raw::{c_int, c_void};

use ffi;
use pyptr::Py;
use python::{Python, ToPythonPointer};
use callback::AbortOnDrop;
use class::NO_METHODS;
use typeob::PyTypeInfo;

pub struct PyTraverseError(c_int);

/// GC support
pub trait PyGCProtocol<'p> : PyTypeInfo {

    fn __traverse__(&'p self, py: Python<'p>, visit: PyVisit) -> Result<(), PyTraverseError>;

    fn __clear__(&'p mut self, py: Python<'p>);

}

impl<'p, T> PyGCProtocol<'p> for T where T: PyTypeInfo {
    default fn __traverse__(&'p self, _py: Python<'p>, _: PyVisit)
                            -> Result<(), PyTraverseError> {
        Ok(())
    }
    default fn __clear__(&'p mut self, _py: Python<'p>) {}
}

#[doc(hidden)]
pub trait PyGCProtocolImpl {
    fn methods() -> &'static [&'static str];

    fn update_type_object(type_object: &mut ffi::PyTypeObject);
}

impl<'p, T> PyGCProtocolImpl for T where T: PyGCProtocol<'p> {
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
        where T: ToPythonPointer
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
    where T: for<'p> PyGCProtocol<'p>
{
    const LOCATION: &'static str = concat!(stringify!(T), ".__traverse__()");

    let guard = AbortOnDrop(LOCATION);
    let py = Python::assume_gil_acquired();
    let visit = PyVisit { visit: visit, arg: arg, _py: py };
    let slf: Py<T> = Py::from_borrowed_ptr(py, slf);

    let ret = match T::__traverse__(&slf, py, visit) {
        Ok(()) => 0,
        Err(PyTraverseError(code)) => code
    };
    mem::forget(guard);
    ret
}

unsafe extern "C" fn tp_clear<T>(slf: *mut ffi::PyObject) -> c_int
    where T: for<'p> PyGCProtocol<'p>
{
    const LOCATION: &'static str = concat!(stringify!(T), ".__clear__()");

    let guard = AbortOnDrop(LOCATION);
    let py = Python::assume_gil_acquired();
    let slf: Py<T> = Py::from_borrowed_ptr(py, slf);
    T::__clear__(slf.as_mut(), py);
    mem::forget(guard);
    0
}
