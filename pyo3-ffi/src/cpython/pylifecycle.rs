use crate::{PyConfig, PyPreConfig, PyStatus, Py_ssize_t};
use libc::wchar_t;
use std::os::raw::{c_char, c_int};

// "private" functions in cpython/pylifecycle.h accepted in PEP 587
extern "C" {
    // skipped _Py_SetStandardStreamEncoding;
    pub fn Py_PreInitialize(src_config: *const PyPreConfig) -> PyStatus;
    pub fn Py_PreInitializeFromBytesArgs(
        src_config: *const PyPreConfig,
        argc: Py_ssize_t,
        argv: *mut *mut c_char,
    ) -> PyStatus;
    pub fn Py_PreInitializeFromArgs(
        src_config: *const PyPreConfig,
        argc: Py_ssize_t,
        argv: *mut *mut wchar_t,
    ) -> PyStatus;
    pub fn _Py_IsCoreInitialized() -> c_int;

    pub fn Py_InitializeFromConfig(config: *const PyConfig) -> PyStatus;
    pub fn _Py_InitializeMain() -> PyStatus;

    pub fn Py_RunMain() -> c_int;

    pub fn Py_ExitStatusException(status: PyStatus) -> !;

    // skipped _Py_RestoreSignals

    // skipped Py_FdIsInteractive
    // skipped _Py_FdIsInteractive

    // skipped _Py_SetProgramFullPath

    // skipped _Py_gitidentifier
    // skipped _Py_getversion

    // skipped _Py_IsFinalizing

    // skipped _PyOS_URandom
    // skipped _PyOS_URandomNonblock

    // skipped _Py_CoerceLegacyLocale
    // skipped _Py_LegacyLocaleDetected
    // skipped _Py_SetLocaleFromEnv

}

#[cfg(Py_3_12)]
pub const PyInterpreterConfig_DEFAULT_GIL: c_int = 0;
#[cfg(Py_3_12)]
pub const PyInterpreterConfig_SHARED_GIL: c_int = 1;
#[cfg(Py_3_12)]
pub const PyInterpreterConfig_OWN_GIL: c_int = 2;

#[cfg(Py_3_12)]
#[repr(C)]
pub struct PyInterpreterConfig {
    pub use_main_obmalloc: c_int,
    pub allow_fork: c_int,
    pub allow_exec: c_int,
    pub allow_threads: c_int,
    pub allow_daemon_threads: c_int,
    pub check_multi_interp_extensions: c_int,
    pub gil: c_int,
}

#[cfg(Py_3_12)]
pub const _PyInterpreterConfig_INIT: PyInterpreterConfig = PyInterpreterConfig {
    use_main_obmalloc: 0,
    allow_fork: 0,
    allow_exec: 0,
    allow_threads: 1,
    allow_daemon_threads: 0,
    check_multi_interp_extensions: 1,
    gil: PyInterpreterConfig_OWN_GIL,
};

#[cfg(Py_3_12)]
pub const _PyInterpreterConfig_LEGACY_INIT: PyInterpreterConfig = PyInterpreterConfig {
    use_main_obmalloc: 1,
    allow_fork: 1,
    allow_exec: 1,
    allow_threads: 1,
    allow_daemon_threads: 1,
    check_multi_interp_extensions: 0,
    gil: PyInterpreterConfig_SHARED_GIL,
};

extern "C" {
    #[cfg(Py_3_12)]
    pub fn Py_NewInterpreterFromConfig(
        tstate_p: *mut *mut crate::PyThreadState,
        config: *const PyInterpreterConfig,
    ) -> PyStatus;
}

// skipped atexit_datacallbackfunc
// skipped _Py_AtExit
