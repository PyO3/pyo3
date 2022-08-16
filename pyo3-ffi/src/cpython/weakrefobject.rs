#[cfg(not(PyPy))]
pub struct _PyWeakReference {
    pub ob_base: crate::PyObject,
    pub wr_object: *mut crate::PyObject,
    pub wr_callback: *mut crate::PyObject,
    pub hash: crate::Py_hash_t,
    pub wr_prev: *mut crate::PyWeakReference,
    pub wr_next: *mut crate::PyWeakReference,
    #[cfg(Py_3_11)]
    pub vectorcall: Option<crate::vectorcallfunc>,
}

// skipped _PyWeakref_GetWeakrefCount
// skipped _PyWeakref_ClearRef
// skipped PyWeakRef_GET_OBJECT
