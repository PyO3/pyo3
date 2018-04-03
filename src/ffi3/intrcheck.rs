use std::os::raw::c_int;

#[cfg_attr(windows, link(name="pythonXY"))] extern "C" {
    #[cfg_attr(PyPy, link_name="\u{1}_PyPyOS_InterruptOccurred")]
    pub fn PyOS_InterruptOccurred() -> c_int;
    pub fn PyOS_InitInterrupts() -> ();
    #[cfg_attr(PyPy, link_name="\u{1}_PyPyOS_AfterFork")]
    pub fn PyOS_AfterFork() -> ();
}