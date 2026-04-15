use crate::object::*;
#[cfg(not(any(Py_LIMITED_API, PyPy, GraalPy)))]
use crate::pyport::Py_hash_t;
use crate::pyport::Py_ssize_t;

pub const PySet_MINSIZE: usize = 8;

#[cfg(not(any(Py_LIMITED_API, PyPy, GraalPy)))]
#[repr(C)]
#[derive(Debug)]
pub struct setentry {
    pub key: *mut PyObject,
    pub hash: Py_hash_t,
}

#[cfg(not(any(Py_LIMITED_API, PyPy, GraalPy)))]
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

// skipped
#[inline]
#[cfg(all(not(any(PyPy, GraalPy)), not(Py_LIMITED_API)))]
pub unsafe fn PySet_GET_SIZE(so: *mut PyObject) -> Py_ssize_t {
    debug_assert_eq!(PyAnySet_Check(so), 1);
    let so = so.cast::<PySetObject>();
    (*so).used
}

// skipped _PySet_Dummy

// Runtime set APIs live in the backend dispatcher.
pub use crate::backend::current::setobject::*;

// skipped _PySet_Update
