use crate::types::{
    PyBool, PyByteArray, PyBytes, PyCapsule, PyComplex, PyDict, PyFloat, PyFrozenSet, PyList,
    PyMapping, PyModule, PySequence, PySet, PySlice, PyString, PyTraceback, PyTuple, PyType,
};
use crate::{ffi, Bound, PyAny, PyMethodDef, PyResult};

use crate::impl_::pymodule::{AddClassToModule, AddTypeToModule, ModuleDef};

use crate::impl_::pyclass::{
    PyClassDictSlot, PyClassDummySlot, PyClassImplCollector, PyClassWeakRefSlot, SendablePyClass,
    ThreadCheckerImpl,
};

pub trait Sealed {}

// for FfiPtrExt
impl Sealed for *mut ffi::PyObject {}

// for PyResultExt
impl Sealed for PyResult<Bound<'_, PyAny>> {}

// for Py(...)Methods
impl Sealed for Bound<'_, PyAny> {}
impl Sealed for Bound<'_, PyBool> {}
impl Sealed for Bound<'_, PyByteArray> {}
impl Sealed for Bound<'_, PyBytes> {}
impl Sealed for Bound<'_, PyCapsule> {}
impl Sealed for Bound<'_, PyComplex> {}
impl Sealed for Bound<'_, PyDict> {}
impl Sealed for Bound<'_, PyFloat> {}
impl Sealed for Bound<'_, PyFrozenSet> {}
impl Sealed for Bound<'_, PyList> {}
impl Sealed for Bound<'_, PyMapping> {}
impl Sealed for Bound<'_, PyModule> {}
impl Sealed for Bound<'_, PySequence> {}
impl Sealed for Bound<'_, PySet> {}
impl Sealed for Bound<'_, PySlice> {}
impl Sealed for Bound<'_, PyString> {}
impl Sealed for Bound<'_, PyTraceback> {}
impl Sealed for Bound<'_, PyTuple> {}
impl Sealed for Bound<'_, PyType> {}

impl<T> Sealed for AddTypeToModule<T> {}
impl<T> Sealed for AddClassToModule<T> {}
impl Sealed for PyMethodDef {}
impl Sealed for ModuleDef {}

impl<T> Sealed for &'_ PyClassImplCollector<T> {}
impl Sealed for PyClassDummySlot {}
impl Sealed for PyClassDictSlot {}
impl Sealed for PyClassWeakRefSlot {}
impl<T: Send> Sealed for SendablePyClass<T> {}
impl Sealed for ThreadCheckerImpl {}
