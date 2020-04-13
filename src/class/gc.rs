// Copyright (c) 2017-present PyO3 Project and Contributors

//! Python GC support
//!

use crate::{ffi, AsPyPointer, PyCell, PyClass, Python};
use std::os::raw::{c_int, c_void};

#[repr(transparent)]
pub struct PyTraverseError(c_int);

/// GC support
pub trait PyGCProtocol<'p>: PyClass {
    fn __traverse__(&'p self, visit: PyVisit) -> Result<(), PyTraverseError>;
    fn __clear__(&'p mut self);
}

pub trait PyGCTraverseProtocol<'p>: PyGCProtocol<'p> {}
pub trait PyGCClearProtocol<'p>: PyGCProtocol<'p> {}

#[doc(hidden)]
pub trait PyGCProtocolImpl {
    fn update_type_object(_type_object: &mut ffi::PyTypeObject);
}

impl<'p, T> PyGCProtocolImpl for T {
    default fn update_type_object(_type_object: &mut ffi::PyTypeObject) {}
}

impl<'p, T> PyGCProtocolImpl for T
where
    T: PyGCProtocol<'p>,
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
    _py: Python<'p>,
}

impl<'p> PyVisit<'p> {
    pub fn call<T>(&self, obj: &T) -> Result<(), PyTraverseError>
    where
        T: AsPyPointer,
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

impl<'p, T> PyGCTraverseProtocolImpl for T
where
    T: PyGCProtocol<'p>,
{
    default fn tp_traverse() -> Option<ffi::traverseproc> {
        None
    }
}

#[doc(hidden)]
impl<T> PyGCTraverseProtocolImpl for T
where
    T: for<'p> PyGCTraverseProtocol<'p>,
{
    #[inline]
    fn tp_traverse() -> Option<ffi::traverseproc> {
        unsafe extern "C" fn tp_traverse<T>(
            slf: *mut ffi::PyObject,
            visit: ffi::visitproc,
            arg: *mut c_void,
        ) -> c_int
        where
            T: for<'p> PyGCTraverseProtocol<'p>,
        {
            let pool = crate::GILPool::new();
            let py = pool.python();
            let slf = py.from_borrowed_ptr::<PyCell<T>>(slf);

            let visit = PyVisit {
                visit,
                arg,
                _py: py,
            };
            let borrow = slf.try_borrow();
            if let Ok(borrow) = borrow {
                match borrow.__traverse__(visit) {
                    Ok(()) => 0,
                    Err(PyTraverseError(code)) => code,
                }
            } else {
                0
            }
        }

        Some(tp_traverse::<T>)
    }
}

trait PyGCClearProtocolImpl {
    fn tp_clear() -> Option<ffi::inquiry>;
}

impl<'p, T> PyGCClearProtocolImpl for T
where
    T: PyGCProtocol<'p>,
{
    default fn tp_clear() -> Option<ffi::inquiry> {
        None
    }
}

impl<T> PyGCClearProtocolImpl for T
where
    T: for<'p> PyGCClearProtocol<'p>,
{
    #[inline]
    fn tp_clear() -> Option<ffi::inquiry> {
        unsafe extern "C" fn tp_clear<T>(slf: *mut ffi::PyObject) -> c_int
        where
            T: for<'p> PyGCClearProtocol<'p>,
        {
            let pool = crate::GILPool::new();
            let py = pool.python();
            let slf = py.from_borrowed_ptr::<PyCell<T>>(slf);

            slf.borrow_mut().__clear__();
            0
        }
        Some(tp_clear::<T>)
    }
}
