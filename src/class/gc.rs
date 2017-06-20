// Copyright (c) 2017-present PyO3 Project and Contributors

//! Python GC support
//!

use std::mem;
use std::os::raw::{c_int, c_void};

use ffi;
use python::{Python, ToPyPointer};
use callback::AbortOnDrop;
use token::{Py, AsPyRef};
use typeob::PyTypeInfo;

pub struct PyTraverseError(c_int);

/// GC support
#[allow(unused_variables)]
pub trait PyGCProtocol<'p> : PyTypeInfo {

    fn __traverse__(&'p self, py: Python<'p>, visit: PyVisit)
                    -> Result<(), PyTraverseError> { unimplemented!() }

    fn __clear__(&'p mut self, py: Python<'p>) { unimplemented!() }

}

pub trait PyGCTraverseProtocol<'p>: PyGCProtocol<'p> {}
pub trait PyGCClearProtocol<'p>: PyGCProtocol<'p> {}


impl<'p, T> PyGCProtocol<'p> for T where T: PyTypeInfo {
    default fn __traverse__(&'p self, _py: Python<'p>, _: PyVisit)
                            -> Result<(), PyTraverseError> {
        Ok(())
    }
    default fn __clear__(&'p mut self, _py: Python<'p>) {}
}

#[doc(hidden)]
pub trait PyGCProtocolImpl {
    fn update_type_object(type_object: &mut ffi::PyTypeObject);
}

impl<'p, T> PyGCProtocolImpl for T where T: PyGCProtocol<'p>
{
    fn update_type_object(type_object: &mut ffi::PyTypeObject) {
        type_object.tp_traverse = Self::tp_traverse();
        type_object.tp_clear = Self::tp_clear();
    }
}

#[derive(Copy, Clone)]
pub struct PyVisit<'p> {
    visit: ffi::visitproc,
    arg: *mut c_void,
    /// VisitProc contains a Python instance to ensure that
    /// 1) it is cannot be moved out of the traverse() call
    /// 2) it cannot be sent to other threads
    _py: Python<'p>
}

impl <'p> PyVisit<'p> {
    pub fn call<T>(&self, obj: &T) -> Result<(), PyTraverseError>
        where T: ToPyPointer
    {
        let r = unsafe { (self.visit)(obj.as_ptr(), self.arg) };
        if r == 0 {
            Ok(())
        } else {
            Err(PyTraverseError(r))
        }
    }
}

trait PyGCTraverseProtocolImpl {
    fn tp_traverse() -> Option<ffi::traverseproc>;
}

impl<'p, T> PyGCTraverseProtocolImpl for T where T: PyGCProtocol<'p>
{
    #[inline]
    default fn tp_traverse() -> Option<ffi::traverseproc> {
        None
    }
}

#[doc(hidden)]
impl<T> PyGCTraverseProtocolImpl for T where T: for<'p> PyGCTraverseProtocol<'p>
{
    #[inline]
    fn tp_traverse() -> Option<ffi::traverseproc> {
        unsafe extern "C" fn tp_traverse<T>(slf: *mut ffi::PyObject,
                                            visit: ffi::visitproc,
                                            arg: *mut c_void) -> c_int
            where T: for<'p> PyGCTraverseProtocol<'p>
        {
            const LOCATION: &'static str = concat!(stringify!(T), ".__traverse__()");

            let guard = AbortOnDrop(LOCATION);
            let py = Python::assume_gil_acquired();
            let visit = PyVisit { visit: visit, arg: arg, _py: py };
            let slf = Py::<T>::from_borrowed_ptr(slf);

            let ret = match slf.as_ref(py).__traverse__(py, visit) {
                Ok(()) => 0,
                Err(PyTraverseError(code)) => code
            };
            mem::forget(guard);
            ret
        }

        Some(tp_traverse::<T>)
    }
}


trait PyGCClearProtocolImpl {
    fn tp_clear() -> Option<ffi::inquiry>;
}

impl<'p, T> PyGCClearProtocolImpl for T where T: PyGCProtocol<'p>
{
    #[inline]
    default fn tp_clear() -> Option<ffi::inquiry> {
        None
    }
}

impl<T> PyGCClearProtocolImpl for T where T: for<'p> PyGCClearProtocol<'p>
{
    #[inline]
    fn tp_clear() -> Option<ffi::inquiry> {
        unsafe extern "C" fn tp_clear<T>(slf: *mut ffi::PyObject) -> c_int
            where T: for<'p> PyGCClearProtocol<'p>
        {
            const LOCATION: &'static str = concat!(stringify!(T), ".__clear__()");

            let guard = AbortOnDrop(LOCATION);
            let py = Python::assume_gil_acquired();
            let slf = Py::<T>::from_borrowed_ptr(slf);
            T::__clear__(&mut slf.as_mut(py), py);
            mem::forget(guard);
            0
        }
        Some(tp_clear::<T>)
    }
}
