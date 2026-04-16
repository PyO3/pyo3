use crate::object::*;

#[cfg(not(Py_LIMITED_API))]
#[repr(C)]
pub struct PySliceObject {
    pub ob_base: PyObject,
    #[cfg(not(GraalPy))]
    pub start: *mut PyObject,
    #[cfg(not(GraalPy))]
    pub stop: *mut PyObject,
    #[cfg(not(GraalPy))]
    pub step: *mut PyObject,
}

pub use crate::backend::current::sliceobject::{
    PyEllipsis_Type, PySlice_AdjustIndices, PySlice_Check, PySlice_GetIndices, PySlice_GetIndicesEx,
    PySlice_New, PySlice_Type, PySlice_Unpack, Py_Ellipsis,
};
