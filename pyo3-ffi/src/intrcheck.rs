use std::os::raw::c_int;

extern "C" {
    #[cfg_attr(PyPy, link_name = "PyPyOS_InterruptOccurred")]
    pub fn PyOS_InterruptOccurred() -> c_int;

    pub fn PyOS_BeforeFork();
    pub fn PyOS_AfterFork_Parent();
    pub fn PyOS_AfterFork_Child();
    #[deprecated(note = "use PyOS_AfterFork_Child instead")]
    #[cfg_attr(PyPy, link_name = "PyPyOS_AfterFork")]
    pub fn PyOS_AfterFork();
}
