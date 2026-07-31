use core::ffi::CStr;

use crate::ffi::PySys_WriteStderr;
use crate::prelude::Python;

pub(crate) fn write_py_stderr(_py: Python<'_>, message: &CStr) {
    // SAFETY: message is a valid c string
    unsafe { PySys_WriteStderr(message.as_ptr()) };
}
