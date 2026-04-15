use crate::{
    err::err_state::{raise_lazy, PyErrStateInner, PyErrStateNormalized},
    ffi,
    ffi_ptr_ext::FfiPtrExt,
    Python,
};

pub(crate) fn fetch(py: Python<'_>) -> Option<PyErrStateNormalized> {
    unsafe { ffi::PyErr_GetRaisedException().assume_owned_or_opt(py) }
        .map(|pvalue| PyErrStateNormalized::new(unsafe { pvalue.cast_into_unchecked() }))
}

pub(crate) fn restore(py: Python<'_>, state: PyErrStateInner) {
    match state {
        PyErrStateInner::Lazy(lazy) => raise_lazy(py, lazy),
        PyErrStateInner::Normalized(normalized) => unsafe {
            ffi::PyErr_SetRaisedException(normalized.into_raised_exception())
        },
    }
}
