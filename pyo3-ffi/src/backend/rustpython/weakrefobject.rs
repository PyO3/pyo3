use crate::object::*;
use crate::pyerrors::PyErr_SetRaisedException;
use crate::rustpython_runtime;
use rustpython_vm::builtins::PyWeak;
use std::ffi::c_int;

pub static mut _PyWeakref_RefType: PyTypeObject = PyTypeObject { _opaque: [] };
pub static mut _PyWeakref_ProxyType: PyTypeObject = PyTypeObject { _opaque: [] };
pub static mut _PyWeakref_CallableProxyType: PyTypeObject = PyTypeObject { _opaque: [] };

#[inline]
pub unsafe fn PyWeakref_CheckRef(op: *mut PyObject) -> c_int {
    if op.is_null() {
        return 0;
    }
    rustpython_runtime::with_vm(|vm| {
        let obj = ptr_to_pyobject_ref_borrowed(op);
        vm.import("_weakref", 0)
            .and_then(|m| m.get_attr("ReferenceType", vm))
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
        vm.import("_weakref", 0)
            .and_then(|m| {
                let proxy = m.get_attr("ProxyType", vm)?;
                let callable = m.get_attr("CallableProxyType", vm)?;
                Ok(obj.class().fast_issubclass(&proxy) || obj.class().fast_issubclass(&callable))
            })
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
        let module = match vm.import("_weakref", 0) {
            Ok(module) => module,
            Err(exc) => {
                PyErr_SetRaisedException(pyobject_ref_to_ptr(exc.into()));
                return std::ptr::null_mut();
            }
        };
        let attr = if is_proxy { "proxy" } else { "ref" };
        let factory = match module.get_attr(attr, vm) {
            Ok(factory) => factory,
            Err(exc) => {
                PyErr_SetRaisedException(pyobject_ref_to_ptr(exc.into()));
                return std::ptr::null_mut();
            }
        };
        let result = if vm.is_none(&callback) {
            factory.call((object,), vm)
        } else {
            factory.call((object, callback), vm)
        };
        match result {
            Ok(value) => pyobject_ref_to_ptr(value),
            Err(exc) => {
                PyErr_SetRaisedException(pyobject_ref_to_ptr(exc.into()));
                std::ptr::null_mut()
            }
        }
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
        if let Some(weak) = reference.downcast_ref::<PyWeak>() {
            return weak.upgrade().map_or_else(
                || pyobject_ref_as_ptr(&vm.ctx.none()),
                |obj| pyobject_ref_as_ptr(&obj),
            );
        }
        if PyWeakref_CheckProxy(reference.as_raw() as *mut PyObject) != 0 {
            return if let Some(method) = reference
                .class()
                .get_attr(vm.ctx.intern_str("__pyo3_referent__"))
            {
                match method.call((reference.to_owned(),), vm) {
                    Ok(obj) => pyobject_ref_as_ptr(&obj),
                    Err(exc) => {
                        PyErr_SetRaisedException(pyobject_ref_to_ptr(exc.into()));
                        std::ptr::null_mut()
                    }
                }
            } else {
                std::ptr::null_mut()
            };
        }
        match reference.call((), vm) {
            Ok(obj) => pyobject_ref_as_ptr(&obj),
            Err(exc) => {
                PyErr_SetRaisedException(pyobject_ref_to_ptr(exc.into()));
                std::ptr::null_mut()
            }
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
        if let Some(weak) = reference.downcast_ref::<PyWeak>() {
            return match weak.upgrade() {
                Some(obj) => {
                    *pobj = pyobject_ref_to_ptr(obj);
                    1
                }
                None => {
                    *pobj = std::ptr::null_mut();
                    0
                }
            };
        }
        if PyWeakref_CheckProxy(reference.as_raw() as *mut PyObject) != 0 {
            return if let Some(method) = reference
                .class()
                .get_attr(vm.ctx.intern_str("__pyo3_referent__"))
            {
                match method.call((reference.to_owned(),), vm) {
                    Ok(obj) => {
                        if vm.is_none(&obj) {
                            *pobj = std::ptr::null_mut();
                            0
                        } else {
                            *pobj = pyobject_ref_to_ptr(obj);
                            1
                        }
                    }
                    Err(exc) => {
                        PyErr_SetRaisedException(pyobject_ref_to_ptr(exc.into()));
                        -1
                    }
                }
            } else {
                PyErr_SetRaisedException(pyobject_ref_to_ptr(
                    vm.new_type_error("weakref proxy missing __pyo3_referent__".to_owned()).into(),
                ));
                -1
            };
        }
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
            Err(exc) => {
                PyErr_SetRaisedException(pyobject_ref_to_ptr(exc.into()));
                -1
            }
        }
    })
}
