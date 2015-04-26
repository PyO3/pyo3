use libc::c_int;
use object::*;

#[link(name = "python2.7")]
extern "C" {
    pub static mut PyRange_Type: PyTypeObject;
}

#[inline(always)]
pub unsafe fn PyRange_Check(op : *mut PyObject) -> c_int {
    let u : *mut PyTypeObject = &mut PyRange_Type;
    (Py_TYPE(op) == u) as c_int
}

