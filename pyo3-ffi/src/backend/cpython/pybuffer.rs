use crate::object::PyObject;
use crate::pybuffer::Py_buffer;
use crate::pyport::Py_ssize_t;
use std::ffi::{c_char, c_int, c_void};

pub(crate) struct HeapTypeBufferView;

pub(crate) enum BufferViewState {
    HeapType(HeapTypeBufferView),
}

/* Return 1 if the getbuffer function is available, otherwise return 0. */
extern_libpython! {
    #[cfg(not(PyPy))]
    pub fn PyObject_CheckBuffer(obj: *mut PyObject) -> c_int;

    #[cfg_attr(PyPy, link_name = "PyPyObject_GetBuffer")]
    pub fn PyObject_GetBuffer(obj: *mut PyObject, view: *mut Py_buffer, flags: c_int) -> c_int;
    #[cfg_attr(PyPy, link_name = "PyPyBuffer_GetPointer")]
    pub fn PyBuffer_GetPointer(view: *const Py_buffer, indices: *const Py_ssize_t) -> *mut c_void;
    #[cfg_attr(PyPy, link_name = "PyPyBuffer_SizeFromFormat")]
    pub fn PyBuffer_SizeFromFormat(format: *const c_char) -> Py_ssize_t;
    #[cfg_attr(PyPy, link_name = "PyPyBuffer_ToContiguous")]
    pub fn PyBuffer_ToContiguous(
        buf: *mut c_void,
        view: *const Py_buffer,
        len: Py_ssize_t,
        order: c_char,
    ) -> c_int;
    #[cfg_attr(PyPy, link_name = "PyPyBuffer_FromContiguous")]
    pub fn PyBuffer_FromContiguous(
        view: *const Py_buffer,
        buf: *const c_void,
        len: Py_ssize_t,
        order: c_char,
    ) -> c_int;
    pub fn PyObject_CopyData(dest: *mut PyObject, src: *mut PyObject) -> c_int;
    #[cfg_attr(PyPy, link_name = "PyPyBuffer_IsContiguous")]
    pub fn PyBuffer_IsContiguous(view: *const Py_buffer, fort: c_char) -> c_int;
    pub fn PyBuffer_FillContiguousStrides(
        ndims: c_int,
        shape: *mut Py_ssize_t,
        strides: *mut Py_ssize_t,
        itemsize: c_int,
        fort: c_char,
    );
    #[cfg_attr(PyPy, link_name = "PyPyBuffer_FillInfo")]
    pub fn PyBuffer_FillInfo(
        view: *mut Py_buffer,
        o: *mut PyObject,
        buf: *mut c_void,
        len: Py_ssize_t,
        readonly: c_int,
        flags: c_int,
    ) -> c_int;
    #[cfg_attr(PyPy, link_name = "PyPyBuffer_Release")]
    pub fn PyBuffer_Release(view: *mut Py_buffer);
}
