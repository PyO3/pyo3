use crate::types::{
    PyBool, PyByteArray, PyBytes, PyCapsule, PyComplex, PyDict, PyFloat, PyFrozenSet, PyList,
    PyMapping, PyMappingProxy, PyModule, PyRange, PySequence, PySet, PySlice, PyString,
    PyTraceback, PyTuple, PyType, PyWeakref, PyWeakrefProxy, PyWeakrefReference,
};
use crate::{ffi, Bound, PyAny, PyResult};

use crate::pyclass_init::PyClassInitializer;

use crate::impl_::{
    pyclass_init::PyNativeTypeInitializer,
    pymethods::PyMethodDef,
    pymodule::{AddClassToModule, AddTypeToModule, ModuleDef},
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
impl Sealed for Bound<'_, PyMappingProxy> {}
impl Sealed for Bound<'_, PyModule> {}
impl Sealed for Bound<'_, PyRange> {}
impl Sealed for Bound<'_, PySequence> {}
impl Sealed for Bound<'_, PySet> {}
impl Sealed for Bound<'_, PySlice> {}
impl Sealed for Bound<'_, PyString> {}
impl Sealed for Bound<'_, PyTraceback> {}
impl Sealed for Bound<'_, PyTuple> {}
impl Sealed for Bound<'_, PyType> {}
impl Sealed for Bound<'_, PyWeakref> {}
impl Sealed for Bound<'_, PyWeakrefProxy> {}
impl Sealed for Bound<'_, PyWeakrefReference> {}

impl<T> Sealed for AddTypeToModule<T> {}
impl<T> Sealed for AddClassToModule<T> {}
impl Sealed for PyMethodDef {}
impl Sealed for ModuleDef {}

impl<T: crate::type_object::PyTypeInfo> Sealed for PyNativeTypeInitializer<T> {}
impl<T: crate::pyclass::PyClass> Sealed for PyClassInitializer<T> {}

impl Sealed for std::sync::Once {}
impl<T> Sealed for std::sync::Mutex<T> {}
#[cfg(feature = "lock_api")]
impl<R, T> Sealed for lock_api::Mutex<R, T> {}
#[cfg(feature = "parking_lot")]
impl Sealed for parking_lot::Once {}
#[cfg(feature = "arc_lock")]
impl<R, T> Sealed for std::sync::Arc<lock_api::Mutex<R, T>> {}
#[cfg(feature = "lock_api")]
impl<R, G, T> Sealed for lock_api::ReentrantMutex<R, G, T> {}
#[cfg(feature = "arc_lock")]
impl<R, G, T> Sealed for std::sync::Arc<lock_api::ReentrantMutex<R, G, T>> {}
