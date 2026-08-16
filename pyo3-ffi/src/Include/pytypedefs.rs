// NB: unlike C, we do not need to forward declare structs in Rust.
// So we only define opaque structs for those which do not have public structure.
//
// Some of these structs defined below only for the opaque PyObject ABI
// (e.g. all(Py_GIL_DISABLED, Py_LIMITED_API)) since we need to define
// the structs but can't know their layout

// PyModuleDef_Slot
// PyMethodDef
// PyGetSetDef
// PyMemberDef

// PyLongObject
opaque_struct!(pub PyCodeObject);
opaque_struct!(pub PyFrameObject);

opaque_struct!(pub PyThreadState);
opaque_struct!(pub PyInterpreterState);
#[cfg(all(Py_LIMITED_API, Py_GIL_DISABLED))]
opaque_struct!(pub PyModuleDef);
#[cfg(Py_LIMITED_API)]
opaque_struct!(pub PyTypeObject);
#[cfg(all(Py_LIMITED_API, Py_GIL_DISABLED))]
opaque_struct!(pub PyObject);
#[cfg(all(Py_LIMITED_API, Py_GIL_DISABLED))]
opaque_struct!(pub PyVarObject);
