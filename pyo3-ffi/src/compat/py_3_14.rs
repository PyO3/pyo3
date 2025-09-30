compat_function!(
    originally_defined_for(all(Py_3_14, not(Py_LIMITED_API)));

    #[inline]
    pub unsafe fn Py_HashBuffer(
        ptr: *const std::ffi::c_void,
        len: crate::Py_ssize_t,
    ) -> crate::Py_hash_t {
        #[cfg(not(any(Py_LIMITED_API, PyPy)))]
        {
            crate::_Py_HashBytes(ptr, len)
        }

        #[cfg(any(Py_LIMITED_API, PyPy))]
        {
            let bytes = crate::PyBytes_FromStringAndSize(ptr as *const std::ffi::c_char, len);
            if bytes.is_null() {
                -1
            } else {
                let result = crate::PyObject_Hash(bytes);
                crate::Py_DECREF(bytes);
                result
            }
        }
    }
);
