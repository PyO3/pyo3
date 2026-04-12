use crate::object::*;
use crate::rustpython_runtime;
use std::ffi::c_int;

pub type PyWeakReference = crate::_PyWeakReference;

#[inline]
pub unsafe fn PyWeakref_CheckRef(op: *mut PyObject) -> c_int {
    if op.is_null() {
        return 0;
    }
    rustpython_runtime::with_vm(|vm| {
        let obj = ptr_to_pyobject_ref_borrowed(op);
        vm.import("weakref", 0)
            .and_then(|m| m.get_attr("ref", vm))
            .map(|ty| obj.class().fast_issubclass(&ty))
            .unwrap_or(false) as c_int
    })
}

#[inline]
pub unsafe fn PyWeakref_CheckRefExact(op: *mut PyObject) -> c_int {
    PyWeakref_CheckRef(op)
}

#[inline]
pub unsafe fn PyWeakref_CheckProxy(op: *mut PyObject) -> c_int {
    if op.is_null() {
        return 0;
    }
    rustpython_runtime::with_vm(|vm| {
        let obj = ptr_to_pyobject_ref_borrowed(op);
        vm.import("weakref", 0)
            .and_then(|m| m.get_attr("ProxyType", vm).or_else(|_| m.get_attr("CallableProxyType", vm)))
            .map(|ty| obj.class().fast_issubclass(&ty))
            .unwrap_or(false) as c_int
    })
}

#[inline]
pub unsafe fn PyWeakref_Check(op: *mut PyObject) -> c_int {
    (PyWeakref_CheckRef(op) != 0 || PyWeakref_CheckProxy(op) != 0) as c_int
}

unsafe fn weakref_call(
    ob: *mut PyObject,
    callback: *mut PyObject,
    is_proxy: bool,
) -> *mut PyObject {
    if ob.is_null() {
        return std::ptr::null_mut();
    }
    rustpython_runtime::with_vm(|vm| {
        let object = ptr_to_pyobject_ref_borrowed(ob);
        let callback = if callback.is_null() {
            vm.ctx.none()
        } else {
            ptr_to_pyobject_ref_borrowed(callback)
        };
        let Ok(module) = vm.import("weakref", 0) else {
            return std::ptr::null_mut();
        };
        let attr = if is_proxy { "proxy" } else { "ref" };
        let Ok(factory) = module.get_attr(attr, vm) else {
            return std::ptr::null_mut();
        };
        let result = if vm.is_none(&callback) {
            factory.call((object,), vm)
        } else {
            factory.call((object, callback), vm)
        };
        result.map(pyobject_ref_to_ptr).unwrap_or(std::ptr::null_mut())
    })
}

#[inline]
pub unsafe fn PyWeakref_NewRef(ob: *mut PyObject, callback: *mut PyObject) -> *mut PyObject {
    weakref_call(ob, callback, false)
}

#[inline]
pub unsafe fn PyWeakref_NewProxy(ob: *mut PyObject, callback: *mut PyObject) -> *mut PyObject {
    weakref_call(ob, callback, true)
}

#[inline]
pub unsafe fn PyWeakref_GetObject(reference: *mut PyObject) -> *mut PyObject {
    if reference.is_null() {
        return std::ptr::null_mut();
    }
    rustpython_runtime::with_vm(|vm| {
        let reference = ptr_to_pyobject_ref_borrowed(reference);
        match reference.call((), vm) {
            Ok(obj) => pyobject_ref_as_ptr(&obj),
            Err(_) => std::ptr::null_mut(),
        }
    })
}

#[cfg(Py_3_13)]
#[inline]
pub unsafe fn PyWeakref_GetRef(reference: *mut PyObject, pobj: *mut *mut PyObject) -> c_int {
    if reference.is_null() || pobj.is_null() {
        return -1;
    }
    rustpython_runtime::with_vm(|vm| {
        let reference = ptr_to_pyobject_ref_borrowed(reference);
        match reference.call((), vm) {
            Ok(obj) => {
                if vm.is_none(&obj) {
                    *pobj = std::ptr::null_mut();
                    0
                } else {
                    *pobj = pyobject_ref_to_ptr(obj);
                    1
                }
            }
            Err(_) => -1,
        }
    })
}
