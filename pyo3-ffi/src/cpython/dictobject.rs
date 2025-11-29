#[cfg(not(GraalPy))]
use crate::object::*;
#[cfg(not(any(PyPy, GraalPy)))]
use crate::pyport::Py_ssize_t;

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

// skipped private _PyDict_GetItem_KnownHash
// skipped private _PyDict_GetItemStringWithError

// skipped PyDict_SetDefault
// skipped PyDict_SetDefaultRef

// skipped PyDict_GET_SIZE
// skipped PyDict_ContainsString

// skipped private _PyDict_NewPresized

// skipped PyDict_Pop
// skipped PyDict_PopString

// skipped private _PyDict_Pop

// skipped PY_FOREACH_DICT_EVENT
// skipped PyDict_WatchEvent

// skipped PyDict_WatchCallback

// skipped PyDict_AddWatcher
// skipped PyDict_ClearWatcher

// skipped PyDict_Watch
// skipped PyDict_Unwatch
