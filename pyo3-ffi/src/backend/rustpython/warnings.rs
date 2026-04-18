use crate::object::*;
use crate::pyerrors::set_vm_exception;
use crate::pyport::Py_ssize_t;
use crate::rustpython_runtime;
use std::ffi::{c_char, c_int, CStr};

#[inline]
pub unsafe fn PyErr_WarnEx(
    category: *mut PyObject,
    message: *const c_char,
    stack_level: Py_ssize_t,
) -> c_int {
    rustpython_runtime::with_vm(|vm| {
        let Ok(warnings) = vm.import("_warnings", 0) else {
            return -1;
        };
        let category = if category.is_null() {
            vm.ctx.none()
        } else {
            ptr_to_pyobject_ref_borrowed(category)
        };
        let message = if message.is_null() {
            String::new()
        } else {
            CStr::from_ptr(message).to_string_lossy().into_owned()
        };
        vm.call_method(
            &warnings,
            "warn",
            (vm.ctx.new_str(message), category, stack_level.max(0) as i32),
        )
        .map(|_| 0)
        .unwrap_or_else(|exc| {
            set_vm_exception(exc);
            -1
        })
    })
}

#[inline]
pub unsafe fn PyErr_WarnExplicit(
    category: *mut PyObject,
    message: *const c_char,
    filename: *const c_char,
    lineno: c_int,
    module: *const c_char,
    registry: *mut PyObject,
) -> c_int {
    rustpython_runtime::with_vm(|vm| {
        let Ok(warnings) = vm.import("_warnings", 0) else {
            return -1;
        };
        let category = if category.is_null() {
            vm.ctx.none()
        } else {
            ptr_to_pyobject_ref_borrowed(category)
        };
        let message = if message.is_null() {
            String::new()
        } else {
            CStr::from_ptr(message).to_string_lossy().into_owned()
        };
        let filename = if filename.is_null() {
            String::new()
        } else {
            CStr::from_ptr(filename).to_string_lossy().into_owned()
        };
        let module = if module.is_null() {
            vm.ctx.none()
        } else {
            vm.ctx
                .new_str(CStr::from_ptr(module).to_string_lossy().into_owned())
                .into()
        };
        let registry = if registry.is_null() {
            vm.ctx.none()
        } else {
            ptr_to_pyobject_ref_borrowed(registry)
        };
        vm.call_method(
            &warnings,
            "warn_explicit",
            (
                vm.ctx.new_str(message),
                category,
                vm.ctx.new_str(filename),
                lineno.max(0) as usize,
                module,
                registry,
            ),
        )
        .map(|_| 0)
        .unwrap_or_else(|exc| {
            set_vm_exception(exc);
            -1
        })
    })
}
