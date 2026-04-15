use crate::{
    err::err_state::{raise_lazy, PyErrStateInner, PyErrStateNormalized},
    ffi,
    ffi_ptr_ext::FfiPtrExt,
    Python,
};

#[cfg(not(Py_3_12))]
use crate::err::err_state::lazy_into_normalized_ffi_tuple;

pub(crate) fn fetch(py: Python<'_>) -> Option<PyErrStateNormalized> {
    #[cfg(Py_3_12)]
    {
        unsafe { ffi::PyErr_GetRaisedException().assume_owned_or_opt(py) }.map(|pvalue| {
            PyErrStateNormalized::new(unsafe { pvalue.cast_into_unchecked() })
        })
    }

    #[cfg(not(Py_3_12))]
    {
        let (ptype, pvalue, ptraceback) = unsafe {
            let mut ptype: *mut ffi::PyObject = std::ptr::null_mut();
            let mut pvalue: *mut ffi::PyObject = std::ptr::null_mut();
            let mut ptraceback: *mut ffi::PyObject = std::ptr::null_mut();

            ffi::PyErr_Fetch(&mut ptype, &mut pvalue, &mut ptraceback);

            if !ptype.is_null() {
                ffi::PyErr_NormalizeException(&mut ptype, &mut pvalue, &mut ptraceback);
            }

            (ptype, pvalue, ptraceback)
        };

        (!ptype.is_null()).then(|| unsafe {
            PyErrStateNormalized::from_normalized_ffi_tuple(py, ptype, pvalue, ptraceback)
        })
    }
}

pub(crate) fn restore(py: Python<'_>, state: PyErrStateInner) {
    #[cfg(Py_3_12)]
    match state {
        PyErrStateInner::Lazy(lazy) => raise_lazy(py, lazy),
        PyErrStateInner::Normalized(normalized) => unsafe {
            ffi::PyErr_SetRaisedException(normalized.into_raised_exception())
        },
    }

    #[cfg(not(Py_3_12))]
    {
        let (ptype, pvalue, ptraceback) = match state {
            PyErrStateInner::Lazy(lazy) => lazy_into_normalized_ffi_tuple(py, lazy),
            PyErrStateInner::Normalized(normalized) => normalized.into_ffi_tuple(py),
        };
        unsafe { ffi::PyErr_Restore(ptype, pvalue, ptraceback) }
    }
}
