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

compat_function!(
    originally_defined_for(Py_3_10);

    #[inline]
    pub unsafe fn PyModule_AddObjectRef(
        module: *mut crate::PyObject,
        name: *const std::ffi::c_char,
        value: *mut crate::PyObject,
    ) -> std::ffi::c_int {
        if value.is_null() && crate::PyErr_Occurred().is_null() {
            crate::PyErr_SetString(
                crate::PyExc_SystemError,
                c_str!("PyModule_AddObjectRef() must be called with an exception raised if value is NULL").as_ptr(),
            );
            return -1;
        }

        crate::Py_XINCREF(value);
        let result = crate::PyModule_AddObject(module, name, value);
        if result < 0 {
            crate::Py_XDECREF(value);
        }
        result
    }
);
