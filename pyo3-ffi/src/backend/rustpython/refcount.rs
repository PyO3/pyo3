use crate::object::PyObject;
use crate::pyport::Py_ssize_t;

#[inline]
pub unsafe fn Py_IncRef(op: *mut PyObject) {
    crate::object::Py_IncRef(op);
}

#[inline]
pub unsafe fn Py_DecRef(op: *mut PyObject) {
    crate::object::Py_DECREF(op);
}

#[inline]
pub unsafe fn Py_REFCNT(ob: *mut PyObject) -> Py_ssize_t {
    if ob.is_null() {
        return 0;
    }
    let obj = unsafe { &*(ob as *mut rustpython_vm::PyObject) };
    obj.strong_count() as Py_ssize_t
}

#[inline]
pub unsafe fn Py_XINCREF(op: *mut PyObject) {
    if !op.is_null() {
        Py_IncRef(op);
    }
}

#[inline]
pub unsafe fn Py_INCREF(op: *mut PyObject) {
    Py_XINCREF(op);
}

#[inline]
pub unsafe fn Py_DECREF(op: *mut PyObject) {
    Py_DecRef(op);
}

#[inline]
pub unsafe fn Py_XDECREF(op: *mut PyObject) {
    if !op.is_null() {
        Py_DECREF(op);
    }
}

#[inline]
pub unsafe fn Py_CLEAR(op: *mut *mut PyObject) {
    if op.is_null() {
        return;
    }
    let tmp = *op;
    *op = std::ptr::null_mut();
    Py_XDECREF(tmp);
}

#[inline]
pub unsafe fn Py_SETREF(op: *mut *mut PyObject, new_ref: *mut PyObject) {
    let old = *op;
    *op = new_ref;
    Py_XDECREF(old);
}

#[inline]
pub unsafe fn Py_XSETREF(op: *mut *mut PyObject, new_ref: *mut PyObject) {
    Py_SETREF(op, new_ref);
}
