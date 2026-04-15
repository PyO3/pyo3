use crate::datetime::{PyDateTime_CAPI, PyDateTime_CAPSULE_NAME};

#[cfg(PyPy)]
use crate::datetime::PyDateTime_Import;

#[cfg(not(PyPy))]
use crate::PyCapsule_Import;

pub unsafe fn import_datetime_api() -> *mut PyDateTime_CAPI {
    #[cfg(PyPy)]
    {
        PyDateTime_Import()
    }

    #[cfg(not(PyPy))]
    {
        PyCapsule_Import(PyDateTime_CAPSULE_NAME.as_ptr(), 1) as *mut PyDateTime_CAPI
    }
}
