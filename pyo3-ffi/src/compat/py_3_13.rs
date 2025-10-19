compat_function!(
    originally_defined_for(Py_3_13);

    #[inline]
    pub unsafe fn PyDict_GetItemRef(
        dp: *mut crate::PyObject,
        key: *mut crate::PyObject,
        result: *mut *mut crate::PyObject,
    ) -> std::ffi::c_int {
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

compat_function!(
    originally_defined_for(Py_3_13);

    #[inline]
    pub unsafe fn PyImport_AddModuleRef(
        name: *const std::ffi::c_char,
    ) -> *mut crate::PyObject {
        use crate::{compat::Py_XNewRef, PyImport_AddModule};

        Py_XNewRef(PyImport_AddModule(name))
    }
);

compat_function!(
    originally_defined_for(Py_3_13);

    #[inline]
    pub unsafe fn PyWeakref_GetRef(
        reference: *mut crate::PyObject,
        pobj: *mut *mut crate::PyObject,
    ) -> std::ffi::c_int {
        use crate::{
            compat::Py_NewRef, PyErr_SetString, PyExc_TypeError, PyWeakref_Check,
            PyWeakref_GetObject, Py_None,
        };

        if !reference.is_null() && PyWeakref_Check(reference) == 0 {
            *pobj = std::ptr::null_mut();
            PyErr_SetString(PyExc_TypeError, c_str!("expected a weakref").as_ptr());
            return -1;
        }
        let obj = PyWeakref_GetObject(reference);
        if obj.is_null() {
            // SystemError if reference is NULL
            *pobj = std::ptr::null_mut();
            return -1;
        }
        if obj == Py_None() {
            *pobj = std::ptr::null_mut();
            return 0;
        }
        *pobj = Py_NewRef(obj);
        1
    }
);

compat_function!(
    originally_defined_for(Py_3_13);

    #[inline]
    pub unsafe fn PyList_Extend(
        list: *mut crate::PyObject,
        iterable: *mut crate::PyObject,
    ) -> std::ffi::c_int {
        crate::PyList_SetSlice(list, crate::PY_SSIZE_T_MAX, crate::PY_SSIZE_T_MAX, iterable)
    }
);

compat_function!(
    originally_defined_for(Py_3_13);

    #[inline]
    pub unsafe fn PyList_Clear(list: *mut crate::PyObject) -> std::ffi::c_int {
        crate::PyList_SetSlice(list, 0, crate::PY_SSIZE_T_MAX, std::ptr::null_mut())
    }
);

compat_function!(
    originally_defined_for(Py_3_13);

    #[inline]
    pub unsafe fn PyModule_Add(
        module: *mut crate::PyObject,
        name: *const std::ffi::c_char,
        value: *mut crate::PyObject,
    ) -> std::ffi::c_int {
        let result = crate::compat::PyModule_AddObjectRef(module, name, value);
        crate::Py_XDECREF(value);
        result
    }
);
