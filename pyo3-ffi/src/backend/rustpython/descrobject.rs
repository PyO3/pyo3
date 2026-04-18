use crate::descrobject::{PyGetSetDef, PyMemberDef};
use crate::methodobject::PyMethodDef;
use crate::object::{PyObject, PyTypeObject};
use crate::rustpython_runtime;
use rustpython_vm::builtins::{PyDict, PyMappingProxy};
use rustpython_vm::{AsObject, PyPayload};
use std::ffi::{c_char, c_int};

pub static mut PyClassMethodDescr_Type: PyTypeObject = PyTypeObject { _opaque: [] };
pub static mut PyGetSetDescr_Type: PyTypeObject = PyTypeObject { _opaque: [] };
pub static mut PyMemberDescr_Type: PyTypeObject = PyTypeObject { _opaque: [] };
pub static mut PyMethodDescr_Type: PyTypeObject = PyTypeObject { _opaque: [] };
pub static mut PyWrapperDescr_Type: PyTypeObject = PyTypeObject { _opaque: [] };
pub static mut PyDictProxy_Type: PyTypeObject = PyTypeObject { _opaque: [] };
pub static mut PyProperty_Type: PyTypeObject = PyTypeObject { _opaque: [] };

#[inline]
pub unsafe fn PyDescr_NewMethod(
    _arg1: *mut PyTypeObject,
    _arg2: *mut PyMethodDef,
) -> *mut PyObject {
    std::ptr::null_mut()
}

#[inline]
pub unsafe fn PyDescr_NewClassMethod(
    _arg1: *mut PyTypeObject,
    _arg2: *mut PyMethodDef,
) -> *mut PyObject {
    std::ptr::null_mut()
}

#[inline]
pub unsafe fn PyDescr_NewMember(
    _arg1: *mut PyTypeObject,
    _arg2: *mut PyMemberDef,
) -> *mut PyObject {
    std::ptr::null_mut()
}

#[inline]
pub unsafe fn PyDescr_NewGetSet(
    _arg1: *mut PyTypeObject,
    _arg2: *mut PyGetSetDef,
) -> *mut PyObject {
    std::ptr::null_mut()
}

#[inline]
pub unsafe fn PyDictProxy_New(arg1: *mut PyObject) -> *mut PyObject {
    if arg1.is_null() {
        return std::ptr::null_mut();
    }
    rustpython_runtime::with_vm(|vm| {
        let mapping = crate::object::ptr_to_pyobject_ref_borrowed(arg1);
        if let Ok(dict) = mapping.clone().downcast::<PyDict>() {
            let proxy = PyMappingProxy::from(dict).into_ref(&vm.ctx);
            return crate::object::pyobject_ref_to_ptr(proxy.into());
        }
        match vm
            .ctx
            .types
            .mappingproxy_type
            .as_object()
            .call((mapping,), vm)
        {
            Ok(proxy) => crate::object::pyobject_ref_to_ptr(proxy),
            Err(_) => std::ptr::null_mut(),
        }
    })
}

#[inline]
pub unsafe fn PyWrapper_New(_arg1: *mut PyObject, _arg2: *mut PyObject) -> *mut PyObject {
    std::ptr::null_mut()
}

#[inline]
pub unsafe fn PyMember_GetOne(_addr: *const c_char, _l: *mut PyMemberDef) -> *mut PyObject {
    std::ptr::null_mut()
}

#[inline]
pub unsafe fn PyMember_SetOne(
    _addr: *mut c_char,
    _l: *mut PyMemberDef,
    _value: *mut PyObject,
) -> c_int {
    -1
}
