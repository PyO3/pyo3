use std::os::raw::c_int;

#[cfg_attr(windows, link(name="pythonXY"))] extern "C" {
    pub fn PyOS_InterruptOccurred() -> c_int;
    pub fn PyOS_InitInterrupts() -> ();
    pub fn PyOS_AfterFork() -> ();
}

