use crate::object::*;
use crate::pyport::Py_ssize_t;
use crate::rustpython_runtime;
use rustpython_vm::AsObject;
use rustpython_vm::builtins::PySlice;
use rustpython_vm::PyPayload;
use rustpython_vm::TryFromBorrowedObject;
use std::ffi::c_int;

pub static mut PySlice_Type: PyTypeObject = PyTypeObject { _opaque: [] };
pub static mut PyEllipsis_Type: PyTypeObject = PyTypeObject { _opaque: [] };

#[inline]
pub unsafe fn Py_Ellipsis() -> *mut PyObject {
    rustpython_runtime::with_vm(|vm| {
        let ellipsis = vm.ctx.ellipsis.clone().into();
        pyobject_ref_as_ptr(&ellipsis)
    })
}

#[cfg(not(Py_LIMITED_API))]
#[repr(C)]
pub struct PySliceObject {
    pub ob_base: PyObject,
    pub start: *mut PyObject,
    pub stop: *mut PyObject,
    pub step: *mut PyObject,
}

#[inline]
pub unsafe fn PySlice_Check(op: *mut PyObject) -> c_int {
    if op.is_null() {
        return 0;
    }
    ptr_to_pyobject_ref_borrowed(op).downcast_ref::<PySlice>().is_some().into()
}

#[inline]
pub unsafe fn PySlice_New(
    start: *mut PyObject,
    stop: *mut PyObject,
    step: *mut PyObject,
) -> *mut PyObject {
    rustpython_runtime::with_vm(|vm| {
        let start = (!start.is_null() && !vm.is_none(&ptr_to_pyobject_ref_borrowed(start)))
            .then(|| ptr_to_pyobject_ref_borrowed(start));
        let stop = if stop.is_null() {
            vm.ctx.none()
        } else {
            ptr_to_pyobject_ref_borrowed(stop)
        };
        let step = (!step.is_null() && !vm.is_none(&ptr_to_pyobject_ref_borrowed(step)))
            .then(|| ptr_to_pyobject_ref_borrowed(step));
        pyobject_ref_to_ptr(PySlice { start, stop, step }.into_ref(&vm.ctx).into())
    })
}

#[inline]
pub unsafe fn PySlice_GetIndices(
    r: *mut PyObject,
    length: Py_ssize_t,
    start: *mut Py_ssize_t,
    stop: *mut Py_ssize_t,
    step: *mut Py_ssize_t,
) -> c_int {
    if PySlice_Unpack(r, start, stop, step) < 0 {
        -1
    } else {
        PySlice_AdjustIndices(length, start, stop, *step);
        0
    }
}

#[inline]
pub unsafe fn PySlice_GetIndicesEx(
    slice: *mut PyObject,
    length: Py_ssize_t,
    start: *mut Py_ssize_t,
    stop: *mut Py_ssize_t,
    step: *mut Py_ssize_t,
    slicelength: *mut Py_ssize_t,
) -> c_int {
    if PySlice_Unpack(slice, start, stop, step) < 0 {
        if !slicelength.is_null() {
            *slicelength = 0;
        }
        -1
    } else {
        if !slicelength.is_null() {
            *slicelength = PySlice_AdjustIndices(length, start, stop, *step);
        }
        0
    }
}

#[inline]
pub unsafe fn PySlice_Unpack(
    slice: *mut PyObject,
    start: *mut Py_ssize_t,
    stop: *mut Py_ssize_t,
    step: *mut Py_ssize_t,
) -> c_int {
    if slice.is_null() {
        return -1;
    }
    let slice = ptr_to_pyobject_ref_borrowed(slice);
    let Some(slice) = slice.downcast_ref::<PySlice>() else {
        return -1;
    };
    rustpython_runtime::with_vm(|vm| {
        let step_obj = slice
            .step
            .as_deref()
            .unwrap_or_else(|| vm.ctx.none.as_object());
        let step_value = match isize::try_from_borrowed_object(vm, step_obj) {
            Ok(v) => v,
            Err(_) if vm.is_none(step_obj) => 1,
            Err(_) => return -1,
        };
        if step_value == 0 {
            return -1;
        }
        let start_obj = slice
            .start
            .as_deref()
            .unwrap_or_else(|| vm.ctx.none.as_object());
        let start_value = match isize::try_from_borrowed_object(vm, start_obj) {
            Ok(v) => v,
            Err(_) if vm.is_none(start_obj) => {
                if step_value < 0 { isize::MAX } else { 0 }
            }
            Err(_) => return -1,
        };
        let stop_obj = slice.stop.as_ref();
        let stop_value = match isize::try_from_borrowed_object(vm, stop_obj) {
            Ok(v) => v,
            Err(_) if vm.is_none(stop_obj) => {
                if step_value < 0 { isize::MIN } else { isize::MAX }
            }
            Err(_) => return -1,
        };
        if !start.is_null() {
            *start = start_value as Py_ssize_t;
        }
        if !stop.is_null() {
            *stop = stop_value as Py_ssize_t;
        }
        if !step.is_null() {
            *step = step_value as Py_ssize_t;
        }
        0
    })
}

#[inline]
pub unsafe fn PySlice_AdjustIndices(
    length: Py_ssize_t,
    start: *mut Py_ssize_t,
    stop: *mut Py_ssize_t,
    step: Py_ssize_t,
) -> Py_ssize_t {
    if start.is_null() || stop.is_null() {
        return 0;
    }
    let len = length.max(0) as usize;
    let step = step as isize;
    if step == 0 {
        return 0;
    }
    let mut start_i = *start as isize;
    let mut stop_i = *stop as isize;
    let len_i = len as isize;
    if step < 0 {
        if start_i >= len_i {
            start_i = len_i - 1;
        } else if start_i < 0 {
            start_i += len_i;
        }
        if stop_i >= len_i {
            stop_i = len_i - 1;
        } else if stop_i < 0 {
            stop_i += len_i;
        }
    } else {
        if start_i < 0 {
            start_i += len_i;
        }
        if stop_i < 0 {
            stop_i += len_i;
        }
        start_i = start_i.clamp(0, len_i);
        stop_i = stop_i.clamp(0, len_i);
    }
    *start = start_i as Py_ssize_t;
    *stop = stop_i as Py_ssize_t;
    if step < 0 {
        if stop_i < start_i {
            ((start_i - stop_i - 1) / (-step) + 1) as Py_ssize_t
        } else {
            0
        }
    } else if start_i < stop_i {
        ((stop_i - start_i - 1) / step + 1) as Py_ssize_t
    } else {
        0
    }
}
