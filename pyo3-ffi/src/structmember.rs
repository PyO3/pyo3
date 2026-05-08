use std::ffi::c_int;

pub use crate::PyMemberDef;

pub use crate::Py_T_BOOL as T_BOOL;
pub use crate::Py_T_BYTE as T_BYTE;
pub use crate::Py_T_CHAR as T_CHAR;
pub use crate::Py_T_DOUBLE as T_DOUBLE;
pub use crate::Py_T_FLOAT as T_FLOAT;
pub use crate::Py_T_INT as T_INT;
pub use crate::Py_T_LONG as T_LONG;
pub use crate::Py_T_LONGLONG as T_LONGLONG;
pub use crate::Py_T_OBJECT_EX as T_OBJECT_EX;
pub use crate::Py_T_SHORT as T_SHORT;
pub use crate::Py_T_STRING as T_STRING;
pub use crate::Py_T_STRING_INPLACE as T_STRING_INPLACE;
pub use crate::Py_T_UBYTE as T_UBYTE;
pub use crate::Py_T_UINT as T_UINT;
pub use crate::Py_T_ULONG as T_ULONG;
pub use crate::Py_T_ULONGLONG as T_ULONGLONG;
pub use crate::Py_T_USHORT as T_USHORT;
#[allow(deprecated)]
pub use crate::_Py_T_OBJECT as T_OBJECT;

pub use crate::Py_T_PYSSIZET as T_PYSSIZET;
#[allow(deprecated)]
pub use crate::_Py_T_NONE as T_NONE;

/* Flags */
pub use crate::Py_READONLY as READONLY;
pub const READ_RESTRICTED: c_int = 2;
pub const PY_WRITE_RESTRICTED: c_int = 4;
pub const RESTRICTED: c_int = READ_RESTRICTED | PY_WRITE_RESTRICTED;
