use crate::object::*;
use std::ffi::c_int;
#[cfg(not(PyPy))]
use std::ptr::addr_of_mut;

#[cfg(all(not(PyPy), Py_LIMITED_API, not(GraalPy)))]
opaque_struct!(pub PyWeakReference);

#[cfg(all(not(PyPy), not(Py_LIMITED_API), not(GraalPy)))]
pub use crate::_PyWeakReference as PyWeakReference;

#[cfg_attr(windows, link(name = "pythonXY"))]
extern "C" {
    // TODO: PyO3 is depending on this symbol in `reference.rs`, we should change this and
    // remove the export as this is a private symbol.
    pub static mut _PyWeakref_RefType: PyTypeObject;
    static mut _PyWeakref_ProxyType: PyTypeObject;
    static mut _PyWeakref_CallableProxyType: PyTypeObject;

    #[cfg(PyPy)]
    #[link_name = "PyPyWeakref_CheckRef"]
    pub fn PyWeakref_CheckRef(op: *mut PyObject) -> c_int;

    #[cfg(PyPy)]
    #[link_name = "PyPyWeakref_CheckRefExact"]
    pub fn PyWeakref_CheckRefExact(op: *mut PyObject) -> c_int;

    #[cfg(PyPy)]
    #[link_name = "PyPyWeakref_CheckProxy"]
    pub fn PyWeakref_CheckProxy(op: *mut PyObject) -> c_int;
}

#[inline]
#[cfg(not(PyPy))]
pub unsafe fn PyWeakref_CheckRef(op: *mut PyObject) -> c_int {
    PyObject_TypeCheck(op, addr_of_mut!(_PyWeakref_RefType))
}

#[inline]
#[cfg(not(PyPy))]
pub unsafe fn PyWeakref_CheckRefExact(op: *mut PyObject) -> c_int {
    (Py_TYPE(op) == addr_of_mut!(_PyWeakref_RefType)) as c_int
}

#[inline]
#[cfg(not(PyPy))]
pub unsafe fn PyWeakref_CheckProxy(op: *mut PyObject) -> c_int {
    ((Py_TYPE(op) == addr_of_mut!(_PyWeakref_ProxyType))
        || (Py_TYPE(op) == addr_of_mut!(_PyWeakref_CallableProxyType))) as c_int
}

#[inline]
pub unsafe fn PyWeakref_Check(op: *mut PyObject) -> c_int {
    (PyWeakref_CheckRef(op) != 0 || PyWeakref_CheckProxy(op) != 0) as c_int
}

extern "C" {
    #[cfg_attr(PyPy, link_name = "PyPyWeakref_NewRef")]
    pub fn PyWeakref_NewRef(ob: *mut PyObject, callback: *mut PyObject) -> *mut PyObject;
    #[cfg_attr(PyPy, link_name = "PyPyWeakref_NewProxy")]
    pub fn PyWeakref_NewProxy(ob: *mut PyObject, callback: *mut PyObject) -> *mut PyObject;
    #[cfg_attr(PyPy, link_name = "PyPyWeakref_GetObject")]
    #[cfg_attr(
        Py_3_13,
        deprecated(note = "deprecated since Python 3.13. Use `PyWeakref_GetRef` instead.")
    )]
    pub fn PyWeakref_GetObject(reference: *mut PyObject) -> *mut PyObject;
    #[cfg(Py_3_13)]
    #[cfg_attr(PyPy, link_name = "PyPyWeakref_GetRef")]
    pub fn PyWeakref_GetRef(reference: *mut PyObject, pobj: *mut *mut PyObject) -> c_int;
}
