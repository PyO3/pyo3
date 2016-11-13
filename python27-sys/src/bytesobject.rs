pub use stringobject::PyStringObject as PyBytesObject;
pub use stringobject::PyString_Type as PyBytes_Type;
pub use stringobject::PyString_Check as PyBytes_Check;
pub use stringobject::PyString_CheckExact as PyBytes_CheckExact;
pub use stringobject::PyString_AS_STRING as PyBytes_AS_STRING;
pub use stringobject::PyString_GET_SIZE as PyBytes_GET_SIZE;
pub use object::Py_TPFLAGS_STRING_SUBCLASS as Py_TPFLAGS_BYTES_SUBCLASS;
pub use stringobject::PyString_FromStringAndSize as PyBytes_FromStringAndSize;
pub use stringobject::PyString_FromString as PyBytes_FromString;
pub use stringobject::PyString_FromFormat as PyBytes_FromFormat;
pub use stringobject::PyString_Size as PyBytes_Size;
pub use stringobject::PyString_AsString as PyBytes_AsString;
pub use stringobject::PyString_Concat as PyBytes_Concat;
pub use stringobject::PyString_ConcatAndDel as PyBytes_ConcatAndDel;
pub use stringobject::PyString_Format as PyBytes_Format;
pub use stringobject::PyString_AsStringAndSize as PyBytes_AsStringAndSize;

