//! C API Compatibility Shims
//!
//! Some CPython C API functions added in recent versions of Python are
//! inherently safer to use than older C API constructs. This module
//! exposes versions of these safer functions for older python versions
//! and on newer versions simply re-exports the function from the FFI
//! bindings.
//!
//! This compatibility module makes it easier to use safer C API
//! constructs without writing your own compatibility shims.

#[cfg(Py_3_13)]
mod py313_compat {
    use crate::dictobject::PyDict_GetItemRef;
}
#[cfg(not(Py_3_13))]
mod py313_compat {
    use crate::object::{PyObject, _Py_NewRef};
    use std::os::raw::c_int;

    use crate::dictobject::PyDict_GetItemWithError;
    use crate::pyerrors::PyErr_Occurred;

    pub unsafe fn PyDict_GetItemRef(
        dp: *mut PyObject,
        key: *mut PyObject,
        result: *mut *mut PyObject,
    ) -> c_int {
        let item: *mut PyObject = PyDict_GetItemWithError(dp, key);
        if !item.is_null() {
            *result = _Py_NewRef(item);
            return 1; // found
        }
        *result = std::ptr::null_mut();
        if PyErr_Occurred().is_null() {
            return 0; // not found
        }
        -1
    }
}

pub use py313_compat::PyDict_GetItemRef;
