// NB: unlike C, we do not need to forward declare structs in Rust.
// So we only define opaque structs for those which do not have public structure.

// PyModuleDef
// PyModuleDef_Slot
// PyMethodDef
// PyGetSetDef
// PyMemberDef

// PyObject
// PyLongObject
// PyTypeObject
opaque_struct!(pub PyCodeObject);
opaque_struct!(pub PyFrameObject);

opaque_struct!(pub PyThreadState);
opaque_struct!(pub PyInterpreterState);
