#[allow(non_snake_case)]
#[inline]
pub unsafe fn PyObject_CallMethodNoArgs(
    obj: *mut crate::PyObject,
    name: *mut crate::PyObject,
) -> *mut crate::PyObject {
    let method = crate::PyObject_GetAttr(obj, name);
    if method.is_null() {
        return std::ptr::null_mut();
    }
    crate::PyObject_CallNoArgs(method)
}
