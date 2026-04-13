use crate::methodobject::PyMethodDef;
use crate::object::{PyObject, PyTypeObject};
use crate::pyport::Py_ssize_t;
use crate::rustpython_runtime;
use rustpython_vm::builtins::{PyDict, PyMappingProxy};
use rustpython_vm::{AsObject, PyPayload};
use std::ffi::{c_char, c_int, c_void};
use std::ptr;

pub type getter = unsafe extern "C" fn(slf: *mut PyObject, closure: *mut c_void) -> *mut PyObject;
pub type setter =
    unsafe extern "C" fn(slf: *mut PyObject, value: *mut PyObject, closure: *mut c_void) -> c_int;

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct PyGetSetDef {
    pub name: *const c_char,
    pub get: Option<getter>,
    pub set: Option<setter>,
    pub doc: *const c_char,
    pub closure: *mut c_void,
}

impl Default for PyGetSetDef {
    fn default() -> Self {
        Self {
            name: ptr::null(),
            get: None,
            set: None,
            doc: ptr::null(),
            closure: ptr::null_mut(),
        }
    }
}

pub static mut PyClassMethodDescr_Type: PyTypeObject = PyTypeObject { _opaque: [] };
pub static mut PyGetSetDescr_Type: PyTypeObject = PyTypeObject { _opaque: [] };
pub static mut PyMemberDescr_Type: PyTypeObject = PyTypeObject { _opaque: [] };
pub static mut PyMethodDescr_Type: PyTypeObject = PyTypeObject { _opaque: [] };
pub static mut PyWrapperDescr_Type: PyTypeObject = PyTypeObject { _opaque: [] };
pub static mut PyDictProxy_Type: PyTypeObject = PyTypeObject { _opaque: [] };
pub static mut PyProperty_Type: PyTypeObject = PyTypeObject { _opaque: [] };

#[inline]
pub unsafe fn PyDescr_NewMethod(_arg1: *mut PyTypeObject, _arg2: *mut PyMethodDef) -> *mut PyObject {
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
pub unsafe fn PyDescr_NewMember(_arg1: *mut PyTypeObject, _arg2: *mut PyMemberDef) -> *mut PyObject {
    std::ptr::null_mut()
}

#[inline]
pub unsafe fn PyDescr_NewGetSet(_arg1: *mut PyTypeObject, _arg2: *mut PyGetSetDef) -> *mut PyObject {
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

#[repr(C)]
#[derive(Copy, Clone, Eq, PartialEq)]
pub struct PyMemberDef {
    pub name: *const c_char,
    pub type_code: c_int,
    pub offset: Py_ssize_t,
    pub flags: c_int,
    pub doc: *const c_char,
}

impl Default for PyMemberDef {
    fn default() -> Self {
        Self {
            name: ptr::null(),
            type_code: 0,
            offset: 0,
            flags: 0,
            doc: ptr::null(),
        }
    }
}

pub const Py_T_SHORT: c_int = 0;
pub const Py_T_INT: c_int = 1;
pub const Py_T_LONG: c_int = 2;
pub const Py_T_FLOAT: c_int = 3;
pub const Py_T_DOUBLE: c_int = 4;
pub const Py_T_STRING: c_int = 5;
pub const _Py_T_OBJECT: c_int = 6;
pub const Py_T_CHAR: c_int = 7;
pub const Py_T_BYTE: c_int = 8;
pub const Py_T_UBYTE: c_int = 9;
pub const Py_T_USHORT: c_int = 10;
pub const Py_T_UINT: c_int = 11;
pub const Py_T_ULONG: c_int = 12;
pub const Py_T_STRING_INPLACE: c_int = 13;
pub const Py_T_BOOL: c_int = 14;
pub const Py_T_OBJECT_EX: c_int = 16;
pub const Py_T_LONGLONG: c_int = 17;
pub const Py_T_ULONGLONG: c_int = 18;
pub const Py_T_PYSSIZET: c_int = 19;
pub const _Py_T_NONE: c_int = 20;

pub const Py_READONLY: c_int = 1;
pub const Py_AUDIT_READ: c_int = 2;
pub const Py_RELATIVE_OFFSET: c_int = 8;
