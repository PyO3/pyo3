#[cfg(all(not(PyPy), Py_LIMITED_API, not(GraalPy)))]
opaque_struct!(pub PyWeakReference);

#[cfg(all(not(PyPy), not(Py_LIMITED_API), not(GraalPy)))]
pub use crate::_PyWeakReference as PyWeakReference;

pub use crate::backend::current::weakrefobject::{
    PyWeakref_Check, PyWeakref_CheckProxy, PyWeakref_CheckRef, PyWeakref_CheckRefExact,
    PyWeakref_GetObject, PyWeakref_NewProxy, PyWeakref_NewRef, _PyWeakref_CallableProxyType,
    _PyWeakref_ProxyType, _PyWeakref_RefType,
};

#[cfg(Py_3_13)]
pub use crate::backend::current::weakrefobject::PyWeakref_GetRef;
