use crate::object::*;
use crate::pyerrors::set_vm_exception;
use crate::rustpython_runtime;
use rustpython_vm::TryFromObject;
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
        let mut traceback = ptr_to_pyobject_ref_borrowed(tb);
        let file = ptr_to_pyobject_ref_borrowed(file);

        let mut rendered = String::from("Traceback (most recent call last):\n");

        loop {
            let frame = match traceback.get_attr("tb_frame", vm) {
                Ok(frame) => frame,
                Err(exc) => {
                    set_vm_exception(exc);
                    return -1;
                }
            };
            let code = match frame.get_attr("f_code", vm) {
                Ok(code) => code,
                Err(exc) => {
                    set_vm_exception(exc);
                    return -1;
                }
            };
            let filename = match code.get_attr("co_filename", vm).and_then(|obj| obj.str(vm)) {
                Ok(filename) => filename,
                Err(exc) => {
                    set_vm_exception(exc);
                    return -1;
                }
            };
            let name = match code.get_attr("co_name", vm).and_then(|obj| obj.str(vm)) {
                Ok(name) => name,
                Err(exc) => {
                    set_vm_exception(exc);
                    return -1;
                }
            };
            let lineno = match traceback
                .get_attr("tb_lineno", vm)
                .and_then(|obj| usize::try_from_object(vm, obj))
            {
                Ok(lineno) => lineno,
                Err(exc) => {
                    set_vm_exception(exc);
                    return -1;
                }
            };

            rendered.push_str("  File \"");
            rendered.push_str(filename.as_ref());
            rendered.push_str("\", line ");
            rendered.push_str(&lineno.to_string());
            rendered.push_str(", in ");
            rendered.push_str(name.as_ref());
            rendered.push('\n');

            let next = match traceback.get_attr("tb_next", vm) {
                Ok(next) => next,
                Err(exc) => {
                    set_vm_exception(exc);
                    return -1;
                }
            };
            if vm.is_none(&next) {
                break;
            }
            traceback = next;
        }

        match vm.call_method(&file, "write", (vm.ctx.new_str(rendered),)) {
            Ok(_) => 0,
            Err(exc) => {
                set_vm_exception(exc);
                -1
            }
        }
    })
}
