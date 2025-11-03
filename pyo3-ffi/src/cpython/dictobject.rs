use crate::object::*;
use crate::pyport::Py_ssize_t;

opaque_struct!(pub PyDictKeysObject);

#[cfg(Py_3_11)]
opaque_struct!(pub PyDictValues);

#[cfg(not(GraalPy))]
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

extern "C" {
    // skipped _PyDict_GetItem_KnownHash
    // skipped _PyDict_GetItemIdWithError
    // skipped _PyDict_GetItemStringWithError
    // skipped PyDict_SetDefault
    // skipped private _PyDict_SetItem_KnownHash
    // skipped _PyDict_DelItem_KnownHash
    // skipped _PyDict_DelItemIf
    // skipped _PyDict_NewKeysForClass
    // skipped _PyDict_Next
    // skipped PyDict_GET_SIZE
    // skipped _PyDict_ContainsId
    // skipped _PyDict_NewPresized
    // skipped _PyDict_MaybeUntrack
    // skipped _PyDict_HasOnlyStringKeys
    // skipped _PyDict_KeysSize
    // skipped _PyDict_SizeOf
    // skipped _PyDict_Pop
    // skipped _PyDict_Pop_KnownHash
    // skipped _PyDict_FromKeys
    // skipped _PyDict_HasSplitTable
    // skipped _PyDict_MergeEx
    // skipped _PyDict_SetItemId
    // skipped _PyDict_DelItemId
    // skipped _PyDict_DebugMallocStats
    // skipped _PyObjectDict_SetItem
    // skipped _PyDict_LoadGlobal
    // skipped _PyDict_GetItemHint
    // skipped _PyDictViewObject
    // skipped _PyDictView_New
    // skipped _PyDictView_Intersect

    // skipped _PyDict_Contains_KnownHash
}
