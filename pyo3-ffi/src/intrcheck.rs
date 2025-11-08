use std::ffi::c_int;

extern "C" {
    #[cfg_attr(PyPy, link_name = "PyPyOS_InterruptOccurred")]
    pub fn PyOS_InterruptOccurred() -> c_int;
    #[cfg(not(Py_3_10))]
    #[deprecated(note = "Not documented in Python API; see Python 3.10 release notes")]
    pub fn PyOS_InitInterrupts();

    pub fn PyOS_BeforeFork();
    pub fn PyOS_AfterFork_Parent();
    pub fn PyOS_AfterFork_Child();
    #[deprecated(note = "use PyOS_AfterFork_Child instead")]
    #[cfg_attr(PyPy, link_name = "PyPyOS_AfterFork")]
    pub fn PyOS_AfterFork();

    // skipped non-limited _PyOS_IsMainThread
    // skipped non-limited Windows _PyOS_SigintEvent
}
