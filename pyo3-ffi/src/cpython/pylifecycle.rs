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

    // skipped Py_ExitStatusException

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

    // skipped _Py_NewInterpreter
}
