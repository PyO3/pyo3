#[allow(non_snake_case)]
#[inline]
pub unsafe fn PyObject_CallMethodNoArgs(
    obj: *mut crate::PyObject,
    name: *mut crate::PyObject,
) -> *mut crate::PyObject {
    crate::PyObject_CallMethodObjArgs(obj, name, std::ptr::null_mut::<crate::PyObject>())
}
