compat_function!(
    originally_defined_for(Py_3_13);

    #[inline]
    pub unsafe fn PyDict_GetItemRef(
        dp: *mut crate::PyObject,
        key: *mut crate::PyObject,
        result: *mut *mut crate::PyObject,
    ) -> std::os::raw::c_int {
        use crate::{compat::Py_NewRef, PyDict_GetItemWithError, PyErr_Occurred};

        let item = PyDict_GetItemWithError(dp, key);
        if !item.is_null() {
            *result = Py_NewRef(item);
            return 1; // found
        }
        *result = std::ptr::null_mut();
        if PyErr_Occurred().is_null() {
            return 0; // not found
        }
        -1
    }
);

compat_function!(
    originally_defined_for(Py_3_13);

    #[inline]
    pub unsafe fn PyList_GetItemRef(
        arg1: *mut crate::PyObject,
        arg2: crate::Py_ssize_t,
    ) -> *mut crate::PyObject {
        use crate::{PyList_GetItem, Py_XINCREF};

        let item = PyList_GetItem(arg1, arg2);
        Py_XINCREF(item);
        item
    }
);
