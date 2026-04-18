use crate::object::*;
use crate::pyerrors::{clear_vm_exception, set_vm_exception};
use crate::pyport::Py_ssize_t;
use crate::rustpython_runtime;
use rustpython_vm::builtins::PyDict;
use rustpython_vm::protocol::PyIterReturn;
use rustpython_vm::PyObjectRef;
use rustpython_vm::{AsObject, PyPayload};
use std::ffi::{c_char, c_int, CStr};

pub static mut PyDict_Type: PyTypeObject = PyTypeObject { _opaque: [] };
pub static mut PyDictKeys_Type: PyTypeObject = PyTypeObject { _opaque: [] };
pub static mut PyDictValues_Type: PyTypeObject = PyTypeObject { _opaque: [] };
pub static mut PyDictItems_Type: PyTypeObject = PyTypeObject { _opaque: [] };
pub static mut PyDictIterKey_Type: PyTypeObject = PyTypeObject { _opaque: [] };
pub static mut PyDictIterValue_Type: PyTypeObject = PyTypeObject { _opaque: [] };
pub static mut PyDictIterItem_Type: PyTypeObject = PyTypeObject { _opaque: [] };
pub static mut PyDictRevIterKey_Type: PyTypeObject = PyTypeObject { _opaque: [] };
pub static mut PyDictRevIterValue_Type: PyTypeObject = PyTypeObject { _opaque: [] };
pub static mut PyDictRevIterItem_Type: PyTypeObject = PyTypeObject { _opaque: [] };

opaque_struct!(pub PyDictObject);

#[inline]
pub unsafe fn PyDict_Check(op: *mut PyObject) -> c_int {
    PyType_FastSubclass(Py_TYPE(op), Py_TPFLAGS_DICT_SUBCLASS)
}

#[inline]
pub unsafe fn PyDict_CheckExact(op: *mut PyObject) -> c_int {
    if op.is_null() {
        return 0;
    }
    ptr_to_pyobject_ref_borrowed(op)
        .downcast_ref::<PyDict>()
        .is_some()
        .into()
}

#[inline]
unsafe fn as_dict(obj: *mut PyObject) -> Option<PyObjectRef> {
    (!obj.is_null()).then(|| ptr_to_pyobject_ref_borrowed(obj))
}

#[inline]
unsafe fn as_dict_exact(obj: *mut PyObject) -> Option<rustpython_vm::PyRef<PyDict>> {
    as_dict(obj)?.downcast::<PyDict>().ok()
}

#[inline]
fn cstr_key(ptr: *const c_char) -> Option<String> {
    (!ptr.is_null()).then(|| {
        unsafe { CStr::from_ptr(ptr) }
            .to_string_lossy()
            .into_owned()
    })
}

#[inline]
pub unsafe fn PyDict_New() -> *mut PyObject {
    rustpython_runtime::with_vm(|vm| pyobject_ref_to_ptr(vm.ctx.new_dict().into()))
}

#[inline]
pub unsafe fn PyDict_GetItem(mp: *mut PyObject, key: *mut PyObject) -> *mut PyObject {
    PyDict_GetItemWithError(mp, key)
}

#[inline]
pub unsafe fn PyDict_GetItemWithError(mp: *mut PyObject, key: *mut PyObject) -> *mut PyObject {
    let Some(dict) = as_dict_exact(mp) else {
        return std::ptr::null_mut();
    };
    let Some(key) = as_dict(key) else {
        return std::ptr::null_mut();
    };
    rustpython_runtime::with_vm(|vm| match dict.get_item_opt(&*key, vm) {
        Ok(Some(value)) => {
            clear_vm_exception();
            pyobject_ref_to_ptr(value)
        }
        Ok(None) => {
            clear_vm_exception();
            std::ptr::null_mut()
        }
        Err(exc) => {
            set_vm_exception(exc);
            std::ptr::null_mut()
        }
    })
}

#[inline]
pub unsafe fn PyDict_SetItem(mp: *mut PyObject, key: *mut PyObject, item: *mut PyObject) -> c_int {
    let Some(dict) = as_dict_exact(mp) else {
        return -1;
    };
    let (Some(key), Some(item)) = (as_dict(key), as_dict(item)) else {
        return -1;
    };
    rustpython_runtime::with_vm(|vm| match dict.set_item(&*key, item, vm) {
        Ok(()) => 0,
        Err(exc) => {
            set_vm_exception(exc);
            -1
        }
    })
}

#[inline]
pub unsafe fn PyDict_DelItem(mp: *mut PyObject, key: *mut PyObject) -> c_int {
    let Some(dict) = as_dict_exact(mp) else {
        return -1;
    };
    let Some(key) = as_dict(key) else {
        return -1;
    };
    rustpython_runtime::with_vm(|vm| match dict.del_item(&*key, vm) {
        Ok(()) => 0,
        Err(exc) => {
            set_vm_exception(exc);
            -1
        }
    })
}

#[inline]
pub unsafe fn PyDict_Clear(mp: *mut PyObject) {
    if let Some(dict) = as_dict_exact(mp) {
        dict.clear();
    }
}

#[inline]
pub unsafe fn PyDict_Next(
    mp: *mut PyObject,
    pos: *mut Py_ssize_t,
    key: *mut *mut PyObject,
    value: *mut *mut PyObject,
) -> c_int {
    let Some(dict) = as_dict_exact(mp) else {
        return 0;
    };
    if pos.is_null() {
        return 0;
    }
    let items = dict.items_vec();
    let current = *pos as usize;
    if current >= items.len() {
        return 0;
    }
    let (k, v) = &items[current];
    if !key.is_null() {
        *key = pyobject_ref_to_ptr(k.clone());
    }
    if !value.is_null() {
        *value = pyobject_ref_to_ptr(v.clone());
    }
    *pos = (current + 1) as Py_ssize_t;
    1
}

#[inline]
pub unsafe fn PyDict_Keys(mp: *mut PyObject) -> *mut PyObject {
    let Some(dict) = as_dict_exact(mp) else {
        return std::ptr::null_mut();
    };
    rustpython_runtime::with_vm(|vm| pyobject_ref_to_ptr(vm.ctx.new_list(dict.keys_vec()).into()))
}

#[inline]
pub unsafe fn PyDict_Values(mp: *mut PyObject) -> *mut PyObject {
    let Some(dict) = as_dict_exact(mp) else {
        return std::ptr::null_mut();
    };
    rustpython_runtime::with_vm(|vm| pyobject_ref_to_ptr(vm.ctx.new_list(dict.values_vec()).into()))
}

#[inline]
pub unsafe fn PyDict_Items(mp: *mut PyObject) -> *mut PyObject {
    let Some(dict) = as_dict_exact(mp) else {
        return std::ptr::null_mut();
    };
    rustpython_runtime::with_vm(|vm| {
        let items = dict
            .items_vec()
            .into_iter()
            .map(|(k, v)| vm.ctx.new_tuple(vec![k, v]).into())
            .collect::<Vec<_>>();
        pyobject_ref_to_ptr(vm.ctx.new_list(items).into())
    })
}

#[inline]
pub unsafe fn PyDict_Size(mp: *mut PyObject) -> Py_ssize_t {
    as_dict_exact(mp)
        .map(|dict| dict.__len__() as Py_ssize_t)
        .unwrap_or(-1)
}

#[inline]
pub unsafe fn PyDict_Copy(mp: *mut PyObject) -> *mut PyObject {
    let Some(dict) = as_dict_exact(mp) else {
        return std::ptr::null_mut();
    };
    rustpython_runtime::with_vm(|vm| pyobject_ref_to_ptr(dict.copy().into_ref(&vm.ctx).into()))
}

#[inline]
pub unsafe fn PyDict_Contains(mp: *mut PyObject, key: *mut PyObject) -> c_int {
    let Some(dict) = as_dict_exact(mp) else {
        return -1;
    };
    let Some(key) = as_dict(key) else {
        return -1;
    };
    rustpython_runtime::with_vm(|vm| dict.contains_key(&*key, vm).into())
}

#[inline]
pub unsafe fn PyDict_Update(mp: *mut PyObject, other: *mut PyObject) -> c_int {
    PyDict_Merge(mp, other, 1)
}

#[inline]
pub unsafe fn PyDict_Merge(mp: *mut PyObject, other: *mut PyObject, _override: c_int) -> c_int {
    let Some(dict) = as_dict_exact(mp) else {
        return -1;
    };
    let Some(other) = as_dict(other) else {
        return -1;
    };
    rustpython_runtime::with_vm(|vm| {
        if _override != 0 {
            return match vm.call_method(dict.as_object(), "update", (other,)) {
                Ok(_) => 0,
                Err(exc) => {
                    set_vm_exception(exc);
                    -1
                }
            };
        }

        let keys = match vm.call_method(&other, "keys", ()) {
            Ok(keys) => keys,
            Err(exc) => {
                set_vm_exception(exc);
                return -1;
            }
        };
        let iter = match keys.get_iter(vm) {
            Ok(iter) => rustpython_vm::protocol::PyIter::new(iter),
            Err(exc) => {
                set_vm_exception(exc);
                return -1;
            }
        };

        loop {
            let key = match iter.next(vm) {
                Ok(PyIterReturn::Return(key)) => key,
                Ok(PyIterReturn::StopIteration(_)) => break,
                Err(exc) => {
                    set_vm_exception(exc);
                    return -1;
                }
            };

            let contains = dict.contains_key(&*key, vm);
            if contains {
                continue;
            }

            let value = match other.get_item(&*key, vm) {
                Ok(value) => value,
                Err(exc) => {
                    set_vm_exception(exc);
                    return -1;
                }
            };

            if let Err(exc) = dict.set_item(&*key, value, vm) {
                set_vm_exception(exc);
                return -1;
            }
        }

        0
    })
}

#[inline]
pub unsafe fn PyDict_MergeFromSeq2(
    d: *mut PyObject,
    seq2: *mut PyObject,
    _override: c_int,
) -> c_int {
    PyDict_Merge(d, seq2, _override)
}

#[inline]
pub unsafe fn PyDict_GetItemString(dp: *mut PyObject, key: *const c_char) -> *mut PyObject {
    let Some(dict) = as_dict_exact(dp) else {
        return std::ptr::null_mut();
    };
    let Some(key) = cstr_key(key) else {
        return std::ptr::null_mut();
    };
    rustpython_runtime::with_vm(|vm| match dict.get_item_opt(key.as_str(), vm) {
        Ok(Some(value)) => {
            clear_vm_exception();
            pyobject_ref_to_ptr(value)
        }
        Ok(None) => {
            clear_vm_exception();
            std::ptr::null_mut()
        }
        Err(exc) => {
            set_vm_exception(exc);
            std::ptr::null_mut()
        }
    })
}

#[inline]
pub unsafe fn PyDict_SetItemString(
    dp: *mut PyObject,
    key: *const c_char,
    item: *mut PyObject,
) -> c_int {
    let Some(dict) = as_dict_exact(dp) else {
        return -1;
    };
    let (Some(key), Some(item)) = (cstr_key(key), as_dict(item)) else {
        return -1;
    };
    rustpython_runtime::with_vm(|vm| match dict.set_item(key.as_str(), item, vm) {
        Ok(()) => 0,
        Err(exc) => {
            set_vm_exception(exc);
            -1
        }
    })
}

#[inline]
pub unsafe fn PyDict_DelItemString(dp: *mut PyObject, key: *const c_char) -> c_int {
    let Some(dict) = as_dict_exact(dp) else {
        return -1;
    };
    let Some(key) = cstr_key(key) else {
        return -1;
    };
    rustpython_runtime::with_vm(|vm| match dict.del_item(key.as_str(), vm) {
        Ok(()) => 0,
        Err(exc) => {
            set_vm_exception(exc);
            -1
        }
    })
}

#[cfg(Py_3_13)]
#[inline]
pub unsafe fn PyDict_GetItemRef(
    dp: *mut PyObject,
    key: *mut PyObject,
    result: *mut *mut PyObject,
) -> c_int {
    let value = PyDict_GetItemWithError(dp, key);
    if !result.is_null() {
        *result = value;
    }
    if !value.is_null() {
        1
    } else if !crate::PyErr_Occurred().is_null() {
        -1
    } else {
        0
    }
}

#[cfg(Py_3_13)]
#[inline]
pub unsafe fn PyDict_GetItemStringRef(
    dp: *mut PyObject,
    key: *const c_char,
    result: *mut *mut PyObject,
) -> c_int {
    let value = PyDict_GetItemString(dp, key);
    if !result.is_null() {
        *result = value;
    }
    (!value.is_null()) as c_int
}

#[inline]
pub unsafe fn PyDictKeys_Check(op: *mut PyObject) -> c_int {
    let Some(obj) = as_dict(op) else {
        return 0;
    };
    rustpython_runtime::with_vm(|vm| {
        let Ok(keys_type) = vm
            .import("builtins", 0)
            .and_then(|m| m.get_attr("dict_keys", vm))
        else {
            return 0;
        };
        obj.is_instance(&keys_type, vm).unwrap_or(false).into()
    })
}

#[inline]
pub unsafe fn PyDictValues_Check(op: *mut PyObject) -> c_int {
    let Some(obj) = as_dict(op) else {
        return 0;
    };
    rustpython_runtime::with_vm(|vm| {
        let Ok(values_type) = vm
            .import("builtins", 0)
            .and_then(|m| m.get_attr("dict_values", vm))
        else {
            return 0;
        };
        obj.is_instance(&values_type, vm).unwrap_or(false).into()
    })
}

#[inline]
pub unsafe fn PyDictItems_Check(op: *mut PyObject) -> c_int {
    let Some(obj) = as_dict(op) else {
        return 0;
    };
    rustpython_runtime::with_vm(|vm| {
        let Ok(items_type) = vm
            .import("builtins", 0)
            .and_then(|m| m.get_attr("dict_items", vm))
        else {
            return 0;
        };
        obj.is_instance(&items_type, vm).unwrap_or(false).into()
    })
}

#[inline]
pub unsafe fn PyDictViewSet_Check(op: *mut PyObject) -> c_int {
    (PyDictKeys_Check(op) != 0 || PyDictItems_Check(op) != 0) as c_int
}
