use crate::object::*;
use crate::pyport::Py_ssize_t;

opaque_struct!(PyDictKeysObject);

#[cfg(Py_3_11)]
opaque_struct!(PyDictValues);

#[cfg(not(GraalPy))]
#[repr(C)]
#[derive(Debug)]
pub struct PyDictObject {
    pub ob_base: PyObject,
    pub ma_used: Py_ssize_t,
    pub ma_version_tag: u64,
    pub ma_keys: *mut PyDictKeysObject,
    #[cfg(not(Py_3_11))]
    pub ma_values: *mut *mut PyObject,
    #[cfg(Py_3_11)]
    pub ma_values: *mut PyDictValues,
}

extern "C" {
    // skipped private _PyDict_GetItem_KnownHash
    // skipped private _PyDict_GetItemStringWithError
    // skipped PyDict_SetDefault
    // skipped PyDict_SetDefaultRef

    // skipped PyDict_GET_SIZE

    // skipped PyDict_ContainsString

    // skipped private _PyDict_NewPresized

    // skipped PyDict_Pop
    // skipped PyDict_PopString
    // skipped _PyDict_Pop

    // skipped PyDict_WatchEvent
    // skipped PyDict_WatchCallback

    // skipped PyDict_AddWatcher
    // skipped PyDict_ClearWatcher

    // skipped PyDict_Watch
    // skipped PyDict_Unwatch
}
