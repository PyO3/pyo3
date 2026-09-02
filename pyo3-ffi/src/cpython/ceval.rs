use crate::cpython::pystate::Py_tracefunc;
use crate::object::{freefunc, PyObject};
use crate::Py_ssize_t;

extern_libpython! {
    pub fn PyEval_SetProfile(trace_func: Option<Py_tracefunc>, arg1: *mut PyObject);
    #[cfg(Py_3_12)]
    pub fn PyEval_SetProfileAllThreads(trace_func: Option<Py_tracefunc>, arg1: *mut PyObject);
    pub fn PyEval_SetTrace(trace_func: Option<Py_tracefunc>, arg1: *mut PyObject);
    #[cfg(Py_3_12)]
    pub fn PyEval_SetTraceAllThreads(trace_func: Option<Py_tracefunc>, arg1: *mut PyObject);

    // skipped PyEval_MergeCompilerFlags

    // skipped private _PyEval_EvalFrameDefault

    // Was moved to the unstable API tier on Py_3_12; older versions export the private name.
    #[cfg_attr(not(Py_3_12), link_name = "_PyEval_RequestCodeExtraIndex")]
    pub fn PyUnstable_Eval_RequestCodeExtraIndex(func: freefunc) -> Py_ssize_t;
}

#[deprecated(
    since = "0.29.0",
    note = "renamed to PyUnstable_Eval_RequestCodeExtraIndex"
)]
#[inline]
pub unsafe extern "C" fn _PyEval_RequestCodeExtraIndex(func: freefunc) -> Py_ssize_t {
    PyUnstable_Eval_RequestCodeExtraIndex(func)
}

extern_libpython! {

    // skipped private _PyEval_SliceIndex
    // skipped private _PyEval_SliceIndexNotNone
    // skipped private _PyEval_UnpackIndices

    // skipped PerfMapState

    // skipped PyUnstable_PerfMapState_Init
    // skipped PyUnstable_WritePerfMapEntry
    // skipped PyUnstable_PerfMapState_Fini
    // skipped PyUnstable_CopyPerfMapFile
    // skipped PyUnstable_PerfTrampoline_CompileCode
    // skipped PyUnstable_PerfTrampoline_SetPersistAfterFork

}
