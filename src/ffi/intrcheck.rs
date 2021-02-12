use std::os::raw::c_int;

extern "C" {
    #[cfg_attr(PyPy, link_name = "PyPyOS_InterruptOccurred")]
    pub fn PyOS_InterruptOccurred() -> c_int;
    #[cfg(not(Py_3_10))]
    #[deprecated(
        since = "0.14.0",
        note = "Not documented in Python API; see Python 3.10 release notes"
    )]
    pub fn PyOS_InitInterrupts();

    #[cfg(any(not(Py_LIMITED_API), Py_3_7))]
    pub fn PyOS_BeforeFork();
    #[cfg(any(not(Py_LIMITED_API), Py_3_7))]
    pub fn PyOS_AfterFork_Parent();
    #[cfg(any(not(Py_LIMITED_API), Py_3_7))]
    pub fn PyOS_AfterFork_Child();
    #[cfg_attr(Py_3_7, deprecated(note = "use PyOS_AfterFork_Child instead"))]
    #[cfg_attr(PyPy, link_name = "PyPyOS_AfterFork")]
    pub fn PyOS_AfterFork();

    // skipped non-limited _PyOS_IsMainThread
    // skipped non-limited Windows _PyOS_SigintEvent
}
