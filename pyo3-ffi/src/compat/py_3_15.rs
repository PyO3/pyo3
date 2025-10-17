#[cfg(not(Py_LIMITED_API))]
compat_function!(
    originally_defined_for(all(Py_3_15, not(Py_LIMITED_API)));

    #[inline]
    pub unsafe fn PyBytesWriter_Create(
        size: crate::Py_ssize_t,
    ) -> *mut crate::PyBytesWriter {

        if size < 0 {
            crate::PyErr_SetString(crate::PyExc_ValueError, c_str!("size must be >= 0").as_ptr() as *const _);
            return std::ptr::null_mut();
        }

        let writer: *mut crate::PyBytesWriter = crate::PyMem_Malloc(std::mem::size_of::<crate::PyBytesWriter>()).cast();
        if writer.is_null() {
            crate::PyErr_NoMemory();
            return std::ptr::null_mut();
        }

        (*writer).obj = std::ptr::null_mut();
        (*writer).size = 0;

        if size >=1 {
            if _PyBytesWriter_Resize_impl(writer, size, 0) < 0 {
                PyBytesWriter_Discard(writer);
                return std::ptr::null_mut();
            }

            (*writer).size = size;
        }

        writer
    }
);

#[cfg(not(Py_LIMITED_API))]
compat_function!(
    originally_defined_for(all(Py_3_15, not(Py_LIMITED_API)));

    #[inline]
    pub unsafe fn PyBytesWriter_Discard(writer: *mut crate::PyBytesWriter) -> () {
        if writer.is_null() {
            return;
        }

        crate::Py_XDECREF((*writer).obj);
        crate::PyMem_Free(writer.cast());
    }
);

#[cfg(not(Py_LIMITED_API))]
compat_function!(
    originally_defined_for(all(Py_3_15, not(Py_LIMITED_API)));

    #[inline]
    pub unsafe fn PyBytesWriter_Finish(writer: *mut crate::PyBytesWriter) -> *mut crate::PyObject {
        PyBytesWriter_FinishWithSize(writer, (*writer).size)
    }
);

#[cfg(not(Py_LIMITED_API))]
compat_function!(
    originally_defined_for(all(Py_3_15, not(Py_LIMITED_API)));

    #[inline]
    pub unsafe fn PyBytesWriter_FinishWithSize(writer: *mut crate::PyBytesWriter, size: crate::Py_ssize_t) -> *mut crate::PyObject {
        let result = if size == 0 {
            crate::PyBytes_FromStringAndSize(c_str!("").as_ptr(), 0)
        } else if (*writer).obj.is_null() {
            crate::PyBytes_FromStringAndSize((*writer).small_buffer.as_ptr(), size)
        } else {
            if size != crate::PyBytes_Size((*writer).obj) && crate::_PyBytes_Resize(&mut (*writer).obj, size) < 0{
                    PyBytesWriter_Discard(writer);
                    return std::ptr::null_mut();
            }
            std::mem::replace(&mut (*writer).obj, std::ptr::null_mut())
        };

        PyBytesWriter_Discard(writer);
        result
    }
);

#[cfg(not(Py_LIMITED_API))]
compat_function!(
    originally_defined_for(all(Py_3_15, not(Py_LIMITED_API)));

    #[inline]
    pub unsafe fn _PyBytesWriter_GetAllocated(writer: *mut crate::PyBytesWriter) -> crate::Py_ssize_t {
        if (*writer).obj.is_null() {
            std::mem::size_of_val(&(*writer).small_buffer) as _
        } else {
            crate::PyBytes_Size((*writer).obj)
        }
    }
);

#[cfg(not(Py_LIMITED_API))]
compat_function!(
    originally_defined_for(all(Py_3_15, not(Py_LIMITED_API)));

    #[inline]
    pub unsafe fn PyBytesWriter_GetData(writer: *mut crate::PyBytesWriter) -> *mut std::ffi::c_void {
        if (*writer).obj.is_null() {
            (*writer).small_buffer.as_ptr() as *mut _
        } else {
                crate::PyBytes_AS_STRING((*writer).obj) as *mut _
        }
    }
);

#[cfg(not(Py_LIMITED_API))]
compat_function!(
    originally_defined_for(all(Py_3_15, not(Py_LIMITED_API)));

    #[inline]
    pub unsafe fn PyBytesWriter_GetSize(writer: *mut crate::PyBytesWriter) -> crate::Py_ssize_t {
        (*writer).size
    }
);

#[cfg(not(Py_LIMITED_API))]
compat_function!(
    originally_defined_for(all(Py_3_15, not(Py_LIMITED_API)));

    #[inline]
    pub unsafe fn PyBytesWriter_WriteBytes(writer: *mut crate::PyBytesWriter, bytes: *const std::ffi::c_void, size: crate::Py_ssize_t) -> std::ffi::c_int {
        let size = if size < 0 {
            let len = libc::strlen(bytes as _);
            if len > crate::PY_SSIZE_T_MAX as libc::size_t {
                crate::PyErr_NoMemory();
                return -1;
            }
            len as crate::Py_ssize_t
        } else {
            size
        };

        let pos = (*writer).size;
        if PyBytesWriter_Grow(writer, size) < 0 {
            return -1;
        }

        let buf = PyBytesWriter_GetData(writer);
        std::ptr::copy_nonoverlapping(bytes, buf.add(pos as usize), size as usize);
        0
    }
);

#[cfg(not(Py_LIMITED_API))]
compat_function!(
    originally_defined_for(all(Py_3_15, not(Py_LIMITED_API)));

    #[inline]
    pub unsafe fn PyBytesWriter_Grow(writer: *mut crate::PyBytesWriter, size: crate::Py_ssize_t) -> std::ffi::c_int {
        if size < 0 && (*writer).size + size < 0 {
            crate::PyErr_SetString(crate::PyExc_ValueError, c_str!("invalid size").as_ptr());
            return -1;
        }

        if size > crate::PY_SSIZE_T_MAX - (*writer).size {
            crate::PyErr_NoMemory();
            return -1;
        }
        let new_size = (*writer).size + size;

        if _PyBytesWriter_Resize_impl(writer, new_size, 1) < 0 {
            return -1;
        }

        (*writer).size = new_size;
        0
    }
);

#[inline]
#[cfg(not(Py_LIMITED_API))]
unsafe fn _PyBytesWriter_Resize_impl(
    writer: *mut crate::PyBytesWriter,
    mut size: crate::Py_ssize_t,
    resize: std::ffi::c_int,
) -> std::ffi::c_int {
    let overallocate = resize;
    assert!(size >= 0);

    if size <= _PyBytesWriter_GetAllocated(writer) {
        return 0;
    }

    if overallocate > 0 {
        #[cfg(windows)]
        if size <= (crate::PY_SSIZE_T_MAX - size / 2) {
            size += size / 2;
        }

        #[cfg(not(windows))]
        if size <= (crate::PY_SSIZE_T_MAX - size / 4) {
            size += size / 4;
        }
    }

    if !(*writer).obj.is_null() {
        if crate::_PyBytes_Resize(&mut (*writer).obj, size) > 0 {
            return -1;
        }
        assert!(!(*writer).obj.is_null())
    } else {
        (*writer).obj = crate::PyBytes_FromStringAndSize(std::ptr::null_mut(), size);
        if (*writer).obj.is_null() {
            return -1;
        }

        if resize > 0 {
            assert!((size as usize) > std::mem::size_of_val(&(*writer).small_buffer));

            std::ptr::copy_nonoverlapping(
                (*writer).small_buffer.as_ptr(),
                crate::PyBytes_AS_STRING((*writer).obj) as *mut _,
                std::mem::size_of_val(&(*writer).small_buffer),
            );
        }
    }

    0
}
