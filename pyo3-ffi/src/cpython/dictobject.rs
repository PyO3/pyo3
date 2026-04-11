#[cfg(not(GraalPy))]
use crate::object::*;
#[cfg(not(any(PyPy, GraalPy)))]
use crate::pyport::Py_ssize_t;

#[cfg(all(not(PyPy), Py_3_13))]
use std::ffi::c_char;
#[cfg(all(not(PyPy), Py_3_12))]
use std::ffi::c_int;

#[cfg(not(PyPy))]
opaque_struct!(pub PyDictKeysObject);

#[cfg(Py_3_11)]
#[cfg(not(PyPy))]
opaque_struct!(pub PyDictValues);

#[cfg(not(any(GraalPy, PyPy)))]
#[repr(C)]
#[derive(Debug)]
pub struct PyDictObject {
    pub ob_base: PyObject,
    pub ma_used: Py_ssize_t,
    #[cfg_attr(
        Py_3_12,
        deprecated(note = "Deprecated in Python 3.12 and will be removed in the future.")
    )]
    #[cfg(not(Py_3_14))]
    pub ma_version_tag: u64,
    #[cfg(Py_3_14)]
    _ma_watcher_tag: u64,
    pub ma_keys: *mut PyDictKeysObject,
    #[cfg(not(Py_3_11))]
    pub ma_values: *mut *mut PyObject,
    #[cfg(Py_3_11)]
    pub ma_values: *mut PyDictValues,
}

#[cfg(PyPy)]
#[repr(C)]
#[derive(Debug)]
pub struct PyDictObject {
    pub ob_base: PyObject,
    _tmpkeys: *mut PyObject,
}

extern_libpython! {
    pub fn PyDict_SetDefault(
        mp: *mut PyObject,
        key: *mut PyObject,
        default_obj: *mut PyObject,
    ) -> *mut PyObject;
    /*
    #[cfg(all(Py_3_13, not(Py_3_15)))]
    pub fn PyDict_SetDefaultRef(
        mp: *mut PyObject,
        key: *mut PyObject,
        default_obj: *mut PyObject,
        result: *mut *mut PyObject,
    ) -> c_int;
    */
    #[cfg(Py_3_13)]
    pub fn PyDict_ContainsString(mp: *mut PyObject, key: *const c_char) -> c_int;
    #[cfg(Py_3_13)]
    pub fn PyDict_Pop(dict: *mut PyObject, key: *mut PyObject, result: *mut *mut PyObject)
        -> c_int;
    #[cfg(Py_3_13)]
    pub fn PyDict_PopString(
        dict: *mut PyObject,
        key: *const c_char,
        result: *mut *mut PyObject,
    ) -> c_int;
    #[cfg(Py_3_12)]
    pub fn PyDict_ClearWatcher(watcher_id: c_int) -> c_int;
    #[cfg(Py_3_12)]
    pub fn PyDict_Watch(watcher_id: c_int, dict: *mut PyObject) -> c_int;
    #[cfg(Py_3_12)]
    pub fn PyDict_Unwatch(watcher_id: c_int, dict: *mut PyObject) -> c_int;
    #[cfg(Py_3_15)]
    pub fn PyFrozenDict_New(iterable: *mut PyObject) -> *mut PyObject;
}

// skipped private _PyDict_GetItem_KnownHash
// skipped private _PyDict_GetItemStringWithError

// skipped PyDict_GET_SIZE

// skipped private _PyDict_NewPresized

// skipped private _PyDict_Pop

// skipped PY_FOREACH_DICT_EVENT
// skipped PyDict_WatchEvent

// skipped PyDict_WatchCallback

// skipped PyDict_AddWatcher
