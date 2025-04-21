compat_function!(
    originally_defined_for(Py_3_10);

    #[inline]
    pub unsafe fn Py_NewRef(obj: *mut crate::PyObject) -> *mut crate::PyObject {
        crate::Py_INCREF(obj);
        obj
    }
);

compat_function!(
    originally_defined_for(Py_3_10);

    #[inline]
    pub unsafe fn Py_XNewRef(obj: *mut crate::PyObject) -> *mut crate::PyObject {
        crate::Py_XINCREF(obj);
        obj
    }
);
