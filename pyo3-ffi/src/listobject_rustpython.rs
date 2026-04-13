use crate::object::*;
use crate::pyerrors::set_vm_exception;
use crate::pyport::Py_ssize_t;
use crate::rustpython_runtime;
use rustpython_vm::builtins::PyList;
use rustpython_vm::function::FuncArgs;
use rustpython_vm::AsObject;
use std::ffi::c_int;

pub static mut PyList_Type: PyTypeObject = PyTypeObject { _opaque: [] };
pub static mut PyListIter_Type: PyTypeObject = PyTypeObject { _opaque: [] };
pub static mut PyListRevIter_Type: PyTypeObject = PyTypeObject { _opaque: [] };

#[inline]
pub unsafe fn PyList_Check(op: *mut PyObject) -> c_int {
    PyType_FastSubclass(Py_TYPE(op), Py_TPFLAGS_LIST_SUBCLASS)
}

#[inline]
pub unsafe fn PyList_CheckExact(op: *mut PyObject) -> c_int {
    if op.is_null() {
        return 0;
    }
    ptr_to_pyobject_ref_borrowed(op)
        .downcast_ref::<PyList>()
        .is_some()
        .into()
}

#[inline]
unsafe fn as_list(obj: *mut PyObject) -> Option<rustpython_vm::PyRef<PyList>> {
    (!obj.is_null())
        .then(|| ptr_to_pyobject_ref_borrowed(obj))
        .and_then(|o| o.downcast::<PyList>().ok())
}

#[inline]
unsafe fn as_obj(obj: *mut PyObject) -> Option<rustpython_vm::PyObjectRef> {
    (!obj.is_null()).then(|| ptr_to_pyobject_ref_borrowed(obj))
}

#[inline]
pub unsafe fn PyList_New(size: Py_ssize_t) -> *mut PyObject {
    rustpython_runtime::with_vm(|vm| {
        let fill = vm.ctx.none();
        let elements = if size <= 0 {
            Vec::new()
        } else {
            vec![fill; size as usize]
        };
        pyobject_ref_to_ptr(vm.ctx.new_list(elements).into())
    })
}

#[inline]
pub unsafe fn PyList_Size(arg1: *mut PyObject) -> Py_ssize_t {
    as_list(arg1)
        .map(|list| list.__len__() as Py_ssize_t)
        .unwrap_or(-1)
}

#[inline]
pub unsafe fn PyList_GetItem(arg1: *mut PyObject, arg2: Py_ssize_t) -> *mut PyObject {
    let Some(list) = as_list(arg1) else {
        return std::ptr::null_mut();
    };
    let vec = list.borrow_vec();
    if arg2 < 0 || (arg2 as usize) >= vec.len() {
        rustpython_runtime::with_vm(|vm| {
            set_vm_exception(vm.new_index_error("list index out of range".to_owned()))
        });
        return std::ptr::null_mut();
    }
    pyobject_ref_to_ptr(vec[arg2 as usize].clone())
}

#[cfg(Py_3_13)]
#[inline]
pub unsafe fn PyList_GetItemRef(arg1: *mut PyObject, arg2: Py_ssize_t) -> *mut PyObject {
    PyList_GetItem(arg1, arg2)
}

#[inline]
pub unsafe fn PyList_SetItem(arg1: *mut PyObject, arg2: Py_ssize_t, arg3: *mut PyObject) -> c_int {
    let Some(list) = as_list(arg1) else {
        return -1;
    };
    let item = if arg3.is_null() {
        rustpython_runtime::with_vm(|vm| vm.ctx.none())
    } else {
        ptr_to_pyobject_ref_owned(arg3)
    };
    let mut elements = list.borrow_vec_mut();
    if arg2 < 0 || (arg2 as usize) >= elements.len() {
        rustpython_runtime::with_vm(|vm| {
            set_vm_exception(vm.new_index_error("list assignment index out of range".to_owned()))
        });
        return -1;
    }
    elements[arg2 as usize] = item;
    0
}

#[inline]
pub unsafe fn PyList_Insert(arg1: *mut PyObject, arg2: Py_ssize_t, arg3: *mut PyObject) -> c_int {
    let Some(list) = as_list(arg1) else {
        return -1;
    };
    let Some(item) = as_obj(arg3) else {
        return -1;
    };
    let mut elements = list.borrow_vec_mut();
    let position = if arg2 < 0 {
        0
    } else {
        (arg2 as usize).min(elements.len())
    };
    elements.insert(position, item);
    0
}

#[inline]
pub unsafe fn PyList_Append(arg1: *mut PyObject, arg2: *mut PyObject) -> c_int {
    let Some(list) = as_list(arg1) else {
        return -1;
    };
    let Some(item) = as_obj(arg2) else {
        return -1;
    };
    list.borrow_vec_mut().push(item);
    0
}

#[inline]
pub unsafe fn PyList_GetSlice(
    arg1: *mut PyObject,
    arg2: Py_ssize_t,
    arg3: Py_ssize_t,
) -> *mut PyObject {
    let Some(list) = as_list(arg1) else {
        return std::ptr::null_mut();
    };
    rustpython_runtime::with_vm(|vm| {
        let vec = list.borrow_vec();
        let start = arg2.max(0) as usize;
        let stop = arg3.max(arg2).max(0) as usize;
        let slice = vec
            .iter()
            .skip(start)
            .take(stop.saturating_sub(start))
            .cloned()
            .collect::<Vec<_>>();
        pyobject_ref_to_ptr(vm.ctx.new_list(slice).into())
    })
}

#[inline]
pub unsafe fn PyList_SetSlice(
    arg1: *mut PyObject,
    arg2: Py_ssize_t,
    arg3: Py_ssize_t,
    arg4: *mut PyObject,
) -> c_int {
    let Some(list) = as_list(arg1) else {
        return -1;
    };
    let replacement = if arg4.is_null() {
        Vec::new()
    } else if let Some(seq) = as_obj(arg4) {
        match rustpython_runtime::with_vm(|vm| seq.try_to_value::<Vec<_>>(vm)) {
            Ok(v) => v,
            Err(exc) => {
                set_vm_exception(exc);
                return -1;
            }
        }
    } else {
        return -1;
    };
    let mut elements = list.borrow_vec_mut();
    let len = elements.len();
    let start = arg2.max(0) as usize;
    let stop = arg3.max(arg2).max(0) as usize;
    if start > len {
        return -1;
    }
    let stop = stop.min(len);
    elements.splice(start..stop, replacement);
    0
}

#[cfg(Py_3_13)]
#[inline]
pub unsafe fn PyList_Extend(list: *mut PyObject, iterable: *mut PyObject) -> c_int {
    let Some(list) = as_list(list) else {
        return -1;
    };
    let Some(iterable) = as_obj(iterable) else {
        return -1;
    };
    rustpython_runtime::with_vm(|vm| match list.extend(iterable, vm) {
        Ok(()) => 0,
        Err(exc) => {
            set_vm_exception(exc);
            -1
        }
    })
}

#[cfg(Py_3_13)]
#[inline]
pub unsafe fn PyList_Clear(list: *mut PyObject) -> c_int {
    let Some(list) = as_list(list) else {
        return -1;
    };
    list.clear();
    0
}

#[inline]
pub unsafe fn PyList_Sort(arg1: *mut PyObject) -> c_int {
    let Some(list) = as_list(arg1) else {
        return -1;
    };
    rustpython_runtime::with_vm(|vm| match vm.call_method(list.as_object(), "sort", FuncArgs::default()) {
        Ok(_) => 0,
        Err(exc) => {
            set_vm_exception(exc);
            -1
        }
    })
}

#[inline]
pub unsafe fn PyList_Reverse(arg1: *mut PyObject) -> c_int {
    let Some(list) = as_list(arg1) else {
        return -1;
    };
    list.borrow_vec_mut().reverse();
    0
}

#[inline]
pub unsafe fn PyList_AsTuple(arg1: *mut PyObject) -> *mut PyObject {
    let Some(list) = as_list(arg1) else {
        return std::ptr::null_mut();
    };
    rustpython_runtime::with_vm(|vm| {
        let items = list.borrow_vec().iter().cloned().collect::<Vec<_>>();
        pyobject_ref_to_ptr(vm.ctx.new_tuple(items).into())
    })
}

#[inline]
pub unsafe fn PyList_GET_ITEM(arg1: *mut PyObject, arg2: Py_ssize_t) -> *mut PyObject {
    let Some(list) = as_list(arg1) else {
        return std::ptr::null_mut();
    };
    let vec = list.borrow_vec();
    if arg2 < 0 || (arg2 as usize) >= vec.len() {
        return std::ptr::null_mut();
    }
    pyobject_ref_as_ptr(&vec[arg2 as usize])
}

#[inline]
pub unsafe fn PyList_GET_SIZE(arg1: *mut PyObject) -> Py_ssize_t {
    PyList_Size(arg1)
}

#[inline]
pub unsafe fn PyList_SET_ITEM(arg1: *mut PyObject, arg2: Py_ssize_t, arg3: *mut PyObject) {
    let _ = PyList_SetItem(arg1, arg2, arg3);
}
