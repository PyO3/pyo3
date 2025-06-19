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
            let bytes = crate::PyBytes_FromStringAndSize(ptr as *const std::os::raw::c_char, len);
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

#[cfg(not(any(Py_LIMITED_API, PyPy)))]
compat_function!(
    originally_defined_for(all(Py_3_14, not(Py_LIMITED_API)));

    pub unsafe fn PyUnicodeWriter_Create(length: crate::Py_ssize_t) -> *mut crate::PyUnicodeWriter {
        if length < 0 {
            crate::PyErr_SetString(
                crate::PyExc_ValueError,
                c_str!("length must be positive").as_ptr(),
            );
            return std::ptr::null_mut();
        }

        let size = std::mem::size_of::<crate::_PyUnicodeWriter>();
        let writer: *mut crate::_PyUnicodeWriter = crate::PyMem_Malloc(size).cast();
        crate::_PyUnicodeWriter_Init(writer);
        if crate::_PyUnicodeWriter_Prepare(writer, length, 127) < 0 {
            PyUnicodeWriter_Discard(writer.cast());
            return std::ptr::null_mut();
        }
        (*writer).overallocate = 1;
        writer.cast()
    }
);

#[cfg(not(any(Py_LIMITED_API, PyPy)))]
compat_function!(
    originally_defined_for(all(Py_3_14, not(Py_LIMITED_API)));

    pub unsafe fn PyUnicodeWriter_Finish(writer: *mut crate::PyUnicodeWriter) -> *mut crate::PyObject {
        let str = crate::_PyUnicodeWriter_Finish(writer.cast());
        crate::PyMem_Free(writer.cast());
        str
    }
);

#[cfg(not(any(Py_LIMITED_API, PyPy)))]
compat_function!(
    originally_defined_for(all(Py_3_14, not(Py_LIMITED_API)));

    pub unsafe fn PyUnicodeWriter_Discard(writer: *mut crate::PyUnicodeWriter) -> () {
        crate::_PyUnicodeWriter_Dealloc(writer.cast());
        crate::PyMem_Free(writer.cast())
    }
);

#[cfg(not(any(Py_LIMITED_API, PyPy)))]
compat_function!(
    originally_defined_for(all(Py_3_14, not(Py_LIMITED_API)));

    pub unsafe fn PyUnicodeWriter_WriteChar(writer: *mut crate::PyUnicodeWriter, ch: crate::Py_UCS4) -> std::os::raw::c_int {
        if ch > 0x10ffff {
            crate::PyErr_SetString(
                crate::PyExc_ValueError,
                c_str!("character must be in range(0x110000)").as_ptr(),
            );
            return -1;
        }

        crate::_PyUnicodeWriter_WriteChar(writer.cast(), ch)
    }
);

#[cfg(not(any(Py_LIMITED_API, PyPy)))]
compat_function!(
    originally_defined_for(all(Py_3_14, not(Py_LIMITED_API)));

    pub unsafe fn PyUnicodeWriter_WriteUTF8(writer: *mut crate::PyUnicodeWriter,str: *const std::os::raw::c_char, size: crate::Py_ssize_t) -> std::os::raw::c_int {
        let size = if size < 0 {
            libc::strlen(str) as isize
        } else {
            size
        };

        let py_str = crate::PyUnicode_FromStringAndSize(str, size);
        if py_str.is_null() {
            return -1;
        }

        let result = crate::_PyUnicodeWriter_WriteStr(writer.cast(), py_str);
        crate::Py_DECREF(py_str);
        result
    }
);
