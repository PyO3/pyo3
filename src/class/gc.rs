// Copyright (c) 2017-present PyO3 Project and Contributors

//! Python GC support
//!

use crate::{derive_utils::TryFromPyCell, ffi, AsPyPointer, PyCell, PyClass, Python};
use std::os::raw::{c_int, c_void};

#[repr(transparent)]
pub struct PyTraverseError(c_int);

/// GC support
#[allow(clippy::upper_case_acronyms)]
pub trait PyGCProtocol<'p>: PyClass {
    fn __traverse__(slf: Self::Receiver, visit: PyVisit) -> Result<(), PyTraverseError>
    where
        Self: PyGCTraverseProtocol<'p>;
    fn __clear__(slf: Self::Receiver)
    where
        Self: PyGCClearProtocol<'p>;
}

#[allow(clippy::upper_case_acronyms)]
pub trait PyGCTraverseProtocol<'p>: PyGCProtocol<'p> {
    type Receiver: TryFromPyCell<'p, Self>;
}

#[allow(clippy::upper_case_acronyms)]
pub trait PyGCClearProtocol<'p>: PyGCProtocol<'p> {
    type Receiver: TryFromPyCell<'p, Self>;
}

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
    let borrow = <T::Receiver as TryFromPyCell<_>>::try_from_pycell(slf);
    if let Ok(borrow) = borrow {
        match T::__traverse__(borrow, visit) {
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

    let borrow = <T::Receiver as TryFromPyCell<_>>::try_from_pycell(slf);

    if let Ok(borrow) = borrow {
        T::__clear__(borrow);
    }
    0
}

/// Object visitor for GC.
#[derive(Clone)]
pub struct PyVisit<'p> {
    visit: ffi::visitproc,
    arg: *mut c_void,
    /// VisitProc contains a Python instance to ensure that
    /// 1) it is cannot be moved out of the traverse() call
    /// 2) it cannot be sent to other threads
    _py: Python<'p>,
}

impl<'p> PyVisit<'p> {
    /// Visit `obj`.
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
