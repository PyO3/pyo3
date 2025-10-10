use crate::object::*;
#[cfg(not(GraalPy))]
use crate::{PyCFunctionObject, PyMethodDefPointer, METH_METHOD, METH_STATIC};
use std::ffi::c_int;
use std::ptr::addr_of_mut;

#[cfg(not(GraalPy))]
pub struct PyCMethodObject {
    pub func: PyCFunctionObject,
    pub mm_class: *mut PyTypeObject,
}

#[cfg_attr(windows, link(name = "pythonXY"))]
extern "C" {
    pub static mut PyCMethod_Type: PyTypeObject;
}

#[inline]
pub unsafe fn PyCMethod_CheckExact(op: *mut PyObject) -> c_int {
    (Py_TYPE(op) == addr_of_mut!(PyCMethod_Type)) as c_int
}

#[inline]
pub unsafe fn PyCMethod_Check(op: *mut PyObject) -> c_int {
    PyObject_TypeCheck(op, addr_of_mut!(PyCMethod_Type))
}

#[cfg(not(GraalPy))]
#[inline]
pub unsafe fn PyCFunction_GET_FUNCTION(func: *mut PyObject) -> PyMethodDefPointer {
    debug_assert_eq!(PyCMethod_Check(func), 1);

    let func = func.cast::<PyCFunctionObject>();
    (*(*func).m_ml).ml_meth
}

#[cfg(not(GraalPy))]
#[inline]
pub unsafe fn PyCFunction_GET_SELF(func: *mut PyObject) -> *mut PyObject {
    debug_assert_eq!(PyCMethod_Check(func), 1);

    let func = func.cast::<PyCFunctionObject>();
    if (*(*func).m_ml).ml_flags & METH_STATIC != 0 {
        std::ptr::null_mut()
    } else {
        (*func).m_self
    }
}

#[cfg(not(GraalPy))]
#[inline]
pub unsafe fn PyCFunction_GET_FLAGS(func: *mut PyObject) -> c_int {
    debug_assert_eq!(PyCMethod_Check(func), 1);

    let func = func.cast::<PyCFunctionObject>();
    (*(*func).m_ml).ml_flags
}

#[cfg(not(GraalPy))]
#[inline]
pub unsafe fn PyCFunction_GET_CLASS(func: *mut PyObject) -> *mut PyTypeObject {
    debug_assert_eq!(PyCMethod_Check(func), 1);

    let func = func.cast::<PyCFunctionObject>();
    if (*(*func).m_ml).ml_flags & METH_METHOD != 0 {
        let func = func.cast::<PyCMethodObject>();
        (*func).mm_class
    } else {
        std::ptr::null_mut()
    }
}
