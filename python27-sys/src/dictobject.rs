use libc::{c_char, c_int};
use pyport::Py_ssize_t;
use object::*;

//pub enum PyDictObject { /* representation hidden */ }

#[cfg_attr(windows, link(name="pythonXY"))] extern "C" {
    pub static mut PyDict_Type: PyTypeObject;
    pub static mut PyDictIterKey_Type: PyTypeObject;
    pub static mut PyDictIterValue_Type: PyTypeObject;
    pub static mut PyDictIterItem_Type: PyTypeObject;
    pub static mut PyDictKeys_Type: PyTypeObject;
    pub static mut PyDictItems_Type: PyTypeObject;
    pub static mut PyDictValues_Type: PyTypeObject;
}

#[inline(always)]
pub unsafe fn PyDict_Check(op : *mut PyObject) -> c_int {
    PyType_FastSubclass(Py_TYPE(op), Py_TPFLAGS_DICT_SUBCLASS)
}

#[inline(always)]
pub unsafe fn PyDict_CheckExact(op : *mut PyObject) -> c_int {
    let u : *mut PyTypeObject = &mut PyDict_Type;
    (Py_TYPE(op) == u) as c_int
}

#[cfg_attr(windows, link(name="pythonXY"))] extern "C" {
    pub fn PyDict_New() -> *mut PyObject;
    pub fn PyDictProxy_New(dict: *mut PyObject) -> *mut PyObject;
    pub fn PyDict_Clear(mp: *mut PyObject);
    pub fn PyDict_Contains(mp: *mut PyObject, key: *mut PyObject)
     -> c_int;
    pub fn PyDict_Copy(mp: *mut PyObject) -> *mut PyObject;
    
    pub fn PyDict_GetItem(mp: *mut PyObject, key: *mut PyObject)
     -> *mut PyObject;
    pub fn PyDict_SetItem(mp: *mut PyObject, key: *mut PyObject,
                          item: *mut PyObject) -> c_int;
    pub fn PyDict_DelItem(mp: *mut PyObject, key: *mut PyObject)
     -> c_int;
    pub fn PyDict_GetItemString(dp: *mut PyObject, key: *const c_char)
     -> *mut PyObject;
    pub fn PyDict_SetItemString(dp: *mut PyObject, key: *const c_char,
                                item: *mut PyObject) -> c_int;
    pub fn PyDict_DelItemString(dp: *mut PyObject, key: *const c_char)
     -> c_int;
    
    pub fn PyDict_Keys(mp: *mut PyObject) -> *mut PyObject;
    pub fn PyDict_Values(mp: *mut PyObject) -> *mut PyObject;
    pub fn PyDict_Items(mp: *mut PyObject) -> *mut PyObject;
    pub fn PyDict_Size(mp: *mut PyObject) -> Py_ssize_t;
    pub fn PyDict_Next(mp: *mut PyObject, pos: *mut Py_ssize_t,
                       key: *mut *mut PyObject, value: *mut *mut PyObject)
     -> c_int;
    /*pub fn _PyDict_Next(mp: *mut PyObject, pos: *mut Py_ssize_t,
                        key: *mut *mut PyObject, value: *mut *mut PyObject,
                        hash: *mut c_long) -> c_int;
    pub fn _PyDict_Contains(mp: *mut PyObject, key: *mut PyObject,
                            hash: c_long) -> c_int;
    pub fn _PyDict_NewPresized(minused: Py_ssize_t) -> *mut PyObject;
    pub fn _PyDict_MaybeUntrack(mp: *mut PyObject);*/
    pub fn PyDict_Update(mp: *mut PyObject, other: *mut PyObject)
     -> c_int;
    pub fn PyDict_Merge(mp: *mut PyObject, other: *mut PyObject,
                        _override: c_int) -> c_int;
    pub fn PyDict_MergeFromSeq2(d: *mut PyObject, seq2: *mut PyObject,
                                _override: c_int) -> c_int;

}

