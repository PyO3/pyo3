compat_function!(
    originally_defined_for(all(
        not(PyPy),
        any(Py_3_10, all(not(Py_LIMITED_API), Py_3_9)) // Added to python in 3.9 but to limited API in 3.10
    ));

    #[inline]
    pub unsafe fn PyObject_CallNoArgs(obj: *mut crate::PyObject) -> *mut crate::PyObject {
        crate::PyObject_CallObject(obj, std::ptr::null_mut())
    }
);

compat_function!(
    originally_defined_for(all(Py_3_9, not(any(Py_LIMITED_API, PyPy))));

    #[inline]
    pub unsafe fn PyObject_CallMethodNoArgs(obj: *mut crate::PyObject, name: *mut crate::PyObject) -> *mut crate::PyObject {
        #[cfg(PyRustPython)]
        {
            let method = crate::PyObject_GetAttr(obj, name);
            if method.is_null() {
                return std::ptr::null_mut();
            }
            return crate::PyObject_CallNoArgs(method);
        }
        #[cfg(not(PyRustPython))]
        crate::PyObject_CallMethodObjArgs(obj, name, std::ptr::null_mut::<crate::PyObject>())
    }
);
