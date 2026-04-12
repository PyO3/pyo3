use crate::object::*;
use crate::rustpython_runtime;

#[inline]
pub unsafe fn PyOS_FSPath(path: *mut PyObject) -> *mut PyObject {
    if path.is_null() {
        return std::ptr::null_mut();
    }
    let path = ptr_to_pyobject_ref_borrowed(path);
    rustpython_runtime::with_vm(|vm| {
        if let Ok(fspath) = path.get_attr("__fspath__", vm).and_then(|meth| meth.call((), vm)) {
            return pyobject_ref_to_ptr(fspath);
        }
        path.str(vm)
            .map(|s| pyobject_ref_to_ptr(s.into()))
            .unwrap_or(std::ptr::null_mut())
    })
}
