use libc::c_int;

extern "C" {
    pub fn PyOS_InterruptOccurred() -> c_int;
    pub fn PyOS_InitInterrupts() -> ();
    pub fn PyOS_AfterFork() -> ();
}

