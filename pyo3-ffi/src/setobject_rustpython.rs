use crate::object::*;
use crate::pyerrors::set_vm_exception;
use crate::pyport::{Py_hash_t, Py_ssize_t};
use crate::rustpython_runtime;
use rustpython_vm::builtins::{PyFrozenSet, PySet};
use rustpython_vm::{AsObject, PyPayload};
use std::ffi::c_int;

pub const PySet_MINSIZE: usize = 8;
pub static mut PySet_Type: PyTypeObject = PyTypeObject { _opaque: [] };
pub static mut PyFrozenSet_Type: PyTypeObject = PyTypeObject { _opaque: [] };
pub static mut PySetIter_Type: PyTypeObject = PyTypeObject { _opaque: [] };

#[inline]
pub unsafe fn PySet_GET_SIZE(so: *mut PyObject) -> Py_ssize_t {
    PySet_Size(so)
}

#[cfg(not(Py_LIMITED_API))]
#[inline]
pub unsafe fn _PySet_NextEntry(
    set: *mut PyObject,
    pos: *mut Py_ssize_t,
    key: *mut *mut PyObject,
    hash: *mut Py_hash_t,
) -> c_int {
    if set.is_null() || pos.is_null() {
        return 0;
    }
    let obj = ptr_to_pyobject_ref_borrowed(set);
    let elements = match obj.downcast_ref::<PySet>() {
        Some(s) => s.elements(),
        None => match obj.downcast_ref::<PyFrozenSet>() {
            Some(s) => s.elements(),
            None => return 0,
        },
    };
    let current = *pos as usize;
    if current >= elements.len() {
        return 0;
    }
    let item = elements[current].clone();
    if !key.is_null() {
        *key = pyobject_ref_to_ptr(item.clone());
    }
    if !hash.is_null() {
        *hash = rustpython_runtime::with_vm(|vm| item.hash(vm).unwrap_or(0) as Py_hash_t);
    }
    *pos = (current + 1) as Py_ssize_t;
    1
}

#[inline]
pub unsafe fn PyFrozenSet_CheckExact(ob: *mut PyObject) -> c_int {
    if ob.is_null() { return 0; }
    ptr_to_pyobject_ref_borrowed(ob).downcast_ref::<PyFrozenSet>().is_some().into()
}

#[inline]
pub unsafe fn PyFrozenSet_Check(ob: *mut PyObject) -> c_int {
    if ob.is_null() {
        return 0;
    }
    let obj = ptr_to_pyobject_ref_borrowed(ob);
    rustpython_runtime::with_vm(|vm| {
        obj.class()
            .fast_issubclass(vm.ctx.types.frozenset_type.as_object()) as c_int
    })
}

#[inline]
pub unsafe fn PyAnySet_CheckExact(ob: *mut PyObject) -> c_int {
    (PySet_CheckExact(ob) != 0 || PyFrozenSet_CheckExact(ob) != 0) as c_int
}

#[inline]
pub unsafe fn PyAnySet_Check(ob: *mut PyObject) -> c_int {
    (PySet_Check(ob) != 0 || PyFrozenSet_Check(ob) != 0) as c_int
}

#[inline]
pub unsafe fn PySet_CheckExact(op: *mut PyObject) -> c_int {
    if op.is_null() { return 0; }
    ptr_to_pyobject_ref_borrowed(op).downcast_ref::<PySet>().is_some().into()
}

#[inline]
pub unsafe fn PySet_Check(ob: *mut PyObject) -> c_int {
    if ob.is_null() {
        return 0;
    }
    let obj = ptr_to_pyobject_ref_borrowed(ob);
    rustpython_runtime::with_vm(|vm| {
        obj.class()
            .fast_issubclass(vm.ctx.types.set_type.as_object()) as c_int
    })
}

#[inline]
pub unsafe fn PySet_New(arg1: *mut PyObject) -> *mut PyObject {
    rustpython_runtime::with_vm(|vm| {
        if arg1.is_null() {
            return pyobject_ref_to_ptr(PySet::default().into_ref(&vm.ctx).into());
        }
        let iterable = ptr_to_pyobject_ref_borrowed(arg1);
        let set = PySet::default().into_ref(&vm.ctx);
        match vm.call_method(set.as_object(), "__ior__", (iterable,)) {
            Ok(_) => pyobject_ref_to_ptr(set.into()),
            Err(exc) => {
                set_vm_exception(exc);
                std::ptr::null_mut()
            }
        }
    })
}

#[inline]
pub unsafe fn PyFrozenSet_New(arg1: *mut PyObject) -> *mut PyObject {
    rustpython_runtime::with_vm(|vm| {
        if arg1.is_null() {
            return pyobject_ref_to_ptr(vm.ctx.empty_frozenset.clone().into());
        }
        let iterable = ptr_to_pyobject_ref_borrowed(arg1);
        let items = match iterable.try_to_value::<Vec<_>>(vm) {
            Ok(items) => items,
            Err(exc) => {
                set_vm_exception(exc);
                return std::ptr::null_mut();
            }
        };
        match PyFrozenSet::from_iter(vm, items) {
            Ok(s) => pyobject_ref_to_ptr(s.into_ref(&vm.ctx).into()),
            Err(exc) => {
                set_vm_exception(exc);
                std::ptr::null_mut()
            }
        }
    })
}

#[inline]
pub unsafe fn PySet_Add(set: *mut PyObject, key: *mut PyObject) -> c_int {
    if set.is_null() || key.is_null() { return -1; }
    let set = ptr_to_pyobject_ref_borrowed(set);
    let key = ptr_to_pyobject_ref_borrowed(key);
    rustpython_runtime::with_vm(|vm| match vm.call_method(&set, "add", (key,)) {
        Ok(_) => 0,
        Err(exc) => {
            set_vm_exception(exc);
            -1
        }
    })
}

#[inline]
pub unsafe fn PySet_Clear(set: *mut PyObject) -> c_int {
    if set.is_null() { return -1; }
    let set = ptr_to_pyobject_ref_borrowed(set);
    rustpython_runtime::with_vm(|vm| match vm.call_method(&set, "clear", ()) {
        Ok(_) => 0,
        Err(exc) => {
            set_vm_exception(exc);
            -1
        }
    })
}

#[inline]
pub unsafe fn PySet_Contains(anyset: *mut PyObject, key: *mut PyObject) -> c_int {
    if anyset.is_null() || key.is_null() { return -1; }
    let set = ptr_to_pyobject_ref_borrowed(anyset);
    let key = ptr_to_pyobject_ref_borrowed(key);
    rustpython_runtime::with_vm(|vm| match vm.call_method(&set, "__contains__", (key,)) {
        Ok(obj) => obj.is(&vm.ctx.true_value).into(),
        Err(exc) => {
            set_vm_exception(exc);
            -1
        }
    })
}

#[inline]
pub unsafe fn PySet_Discard(set: *mut PyObject, key: *mut PyObject) -> c_int {
    if set.is_null() || key.is_null() { return -1; }
    let set = ptr_to_pyobject_ref_borrowed(set);
    let key = ptr_to_pyobject_ref_borrowed(key);
    rustpython_runtime::with_vm(|vm| {
        let present = match vm.call_method(&set, "__contains__", (key.clone(),)) {
            Ok(obj) => obj.is(&vm.ctx.true_value),
            Err(exc) => {
                set_vm_exception(exc);
                return -1;
            }
        };
        if !present {
            return 0;
        }
        match vm.call_method(&set, "discard", (key,)) {
            Ok(_) => 1,
            Err(exc) => {
                set_vm_exception(exc);
                -1
            }
        }
    })
}

#[inline]
pub unsafe fn PySet_Pop(set: *mut PyObject) -> *mut PyObject {
    if set.is_null() { return std::ptr::null_mut(); }
    let set = ptr_to_pyobject_ref_borrowed(set);
    rustpython_runtime::with_vm(|vm| match vm.call_method(&set, "pop", ()) {
        Ok(obj) => pyobject_ref_to_ptr(obj),
        Err(exc) => {
            set_vm_exception(exc);
            std::ptr::null_mut()
        }
    })
}

#[inline]
pub unsafe fn PySet_Size(anyset: *mut PyObject) -> Py_ssize_t {
    if anyset.is_null() { return -1; }
    let obj = ptr_to_pyobject_ref_borrowed(anyset);
    match obj.downcast_ref::<PySet>() {
        Some(s) => s.elements().len() as Py_ssize_t,
        None => match obj.downcast_ref::<PyFrozenSet>() {
            Some(s) => s.elements().len() as Py_ssize_t,
            None => -1,
        },
    }
}
