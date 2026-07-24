#[cfg(Py_GIL_DISABLED)]
use crate::pyatomic::_Py_atomic_load_ssize_relaxed;
#[cfg(not(any(PyPy, GraalPy)))]
use crate::{PyAnySet_Check, PyObject, Py_hash_t, Py_ssize_t};

pub const PySet_MINSIZE: usize = 8;

#[cfg(not(any(PyPy, GraalPy)))]
#[repr(C)]
#[derive(Debug)]
pub struct setentry {
    pub key: *mut PyObject,
    pub hash: Py_hash_t,
}

#[cfg(not(any(PyPy, GraalPy)))]
#[repr(C)]
#[derive(Debug)]
pub struct PySetObject {
    pub ob_base: PyObject,
    pub fill: Py_ssize_t,
    pub used: Py_ssize_t,
    pub mask: Py_ssize_t,
    pub table: *mut setentry,
    pub hash: Py_hash_t,
    pub finger: Py_ssize_t,
    pub smalltable: [setentry; PySet_MINSIZE],
    pub weakreflist: *mut PyObject,
}

#[inline]
#[cfg(not(any(PyPy, GraalPy)))]
pub(crate) unsafe fn _PySet_CAST(so: *mut PyObject) -> *mut PySetObject {
    debug_assert_eq!(PyAnySet_Check(so), 1);
    so.cast()
}

#[inline]
#[cfg(not(any(PyPy, GraalPy)))]
pub unsafe fn PySet_GET_SIZE(so: *mut PyObject) -> Py_ssize_t {
    let so = _PySet_CAST(so);
    #[cfg(Py_GIL_DISABLED)]
    {
        _Py_atomic_load_ssize_relaxed(&raw const (*so).used)
    }
    #[cfg(not(Py_GIL_DISABLED))]
    {
        (*so).used
    }
}
