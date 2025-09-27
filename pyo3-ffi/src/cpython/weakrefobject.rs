// NB publicly re-exported in `src/weakrefobject.rs`
#[cfg(not(any(PyPy, GraalPy)))]
pub struct _PyWeakReference {
    pub ob_base: crate::PyObject,
    pub wr_object: *mut crate::PyObject,
    pub wr_callback: *mut crate::PyObject,
    pub hash: crate::Py_hash_t,
    pub wr_prev: *mut crate::PyWeakReference,
    pub wr_next: *mut crate::PyWeakReference,
    #[cfg(Py_3_11)]
    pub vectorcall: Option<crate::vectorcallfunc>,
    #[cfg(all(Py_3_13, Py_GIL_DISABLED))]
    pub weakrefs_lock: *mut crate::PyMutex,
}

// skipped _PyWeakref_GetWeakrefCount
// skipped _PyWeakref_ClearRef
// skipped PyWeakRef_GET_OBJECT
