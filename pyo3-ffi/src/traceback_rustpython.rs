use crate::object::*;
use crate::rustpython_runtime;
use std::ffi::c_int;

#[inline]
pub unsafe fn PyTraceBack_Check(op: *mut PyObject) -> c_int {
    if op.is_null() {
        return 0;
    }
    rustpython_runtime::with_vm(|vm| {
        let obj = ptr_to_pyobject_ref_borrowed(op);
        vm.import("types", 0)
            .and_then(|m| m.get_attr("TracebackType", vm))
            .map(|ty| obj.class().fast_issubclass(&ty))
            .unwrap_or(false) as c_int
    })
}

#[inline]
pub unsafe fn PyTraceBack_Print(tb: *mut PyObject, file: *mut PyObject) -> c_int {
    if tb.is_null() || file.is_null() {
        return -1;
    }
    rustpython_runtime::with_vm(|vm| {
        let traceback = ptr_to_pyobject_ref_borrowed(tb);
        let file = ptr_to_pyobject_ref_borrowed(file);
        let Ok(mod_) = vm.import("traceback", 0) else {
            return -1;
        };
        let Ok(text) = vm.call_method(&mod_, "format_tb", (traceback,)) else {
            return -1;
        };
        match vm.call_method(&file, "write", (text,)) {
            Ok(_) => 0,
            Err(_) => -1,
        }
    })
}
