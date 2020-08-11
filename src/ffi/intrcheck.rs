use std::os::raw::c_int;

extern "C" {
    #[cfg_attr(PyPy, link_name = "PyPyOS_InterruptOccurred")]
    pub fn PyOS_InterruptOccurred() -> c_int;
    pub fn PyOS_InitInterrupts();
    #[cfg_attr(PyPy, link_name = "PyPyOS_AfterFork")]
    pub fn PyOS_AfterFork();
}
