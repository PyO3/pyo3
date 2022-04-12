#![allow(deprecated)]
// Copyright (c) 2017-present PyO3 Project and Contributors

//! Python GC support

use crate::{ffi, pyclass::MutablePyClass, PyCell};
use std::os::raw::{c_int, c_void};

pub use crate::impl_::pymethods::{PyTraverseError, PyVisit};

/// GC support
#[deprecated(since = "0.16.0", note = "prefer `#[pymethods]` to `#[pyproto]`")]
pub trait PyGCProtocol<'p>: MutablePyClass {
    fn __traverse__(&'p self, visit: PyVisit<'_>) -> Result<(), PyTraverseError>;
    fn __clear__(&'p mut self);
}

pub trait PyGCTraverseProtocol<'p>: PyGCProtocol<'p> {}

pub trait PyGCClearProtocol<'p>: PyGCProtocol<'p> {}

#[doc(hidden)]
pub unsafe extern "C" fn traverse<T>(
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

#[doc(hidden)]
pub unsafe extern "C" fn clear<T>(slf: *mut ffi::PyObject) -> c_int
where
    T: for<'p> PyGCClearProtocol<'p>,
{
    let pool = crate::GILPool::new();
    let slf = pool.python().from_borrowed_ptr::<PyCell<T>>(slf);

    slf.borrow_mut().__clear__();
    0
}
