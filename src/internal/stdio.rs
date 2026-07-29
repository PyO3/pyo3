use core::ffi::CStr;

use crate::ffi::PySys_WriteStderr;

pub(crate) fn write_py_stderr(message: &CStr) {
    // SAFETY: message is a valid c string
    unsafe { PySys_WriteStderr(message.as_ptr()) };
}
