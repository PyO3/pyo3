use std::os::raw::{c_int, c_void};

use crate::{ffi, AsPyPointer, Python};

/// Error returned by a `__traverse__` visitor implementation.
#[repr(transparent)]
pub struct PyTraverseError(pub(crate) c_int);

/// Object visitor for GC.
#[derive(Clone)]
pub struct PyVisit<'p> {
    pub(crate) visit: ffi::visitproc,
    pub(crate) arg: *mut c_void,
    /// VisitProc contains a Python instance to ensure that
    /// 1) it is cannot be moved out of the traverse() call
    /// 2) it cannot be sent to other threads
    pub(crate) _py: Python<'p>,
}

impl<'p> PyVisit<'p> {
    /// Visit `obj`.
    pub fn call<T>(&self, obj: &T) -> Result<(), PyTraverseError>
    where
        T: AsPyPointer,
    {
        let ptr = obj.as_ptr();
        if !ptr.is_null() {
            let r = unsafe { (self.visit)(ptr, self.arg) };
            if r == 0 {
                Ok(())
            } else {
                Err(PyTraverseError(r))
            }
        } else {
            Ok(())
        }
    }

    /// Creates the PyVisit from the arguments to tp_traverse
    #[doc(hidden)]
    pub unsafe fn from_raw(visit: ffi::visitproc, arg: *mut c_void, py: Python<'p>) -> Self {
        Self {
            visit,
            arg,
            _py: py,
        }
    }
}
