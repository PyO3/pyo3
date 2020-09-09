// Copyright (c) 2017-present PyO3 Project and Contributors

use crate::{ffi, PyObject, Python};
use libc::c_int;
use std::ffi::{CStr, CString};
use std::fmt;

/// `PyMethodDefType` represents different types of Python callable objects.
/// It is used by the `#[pymethods]` and `#[pyproto]` annotations.
#[derive(Debug)]
pub enum PyMethodDefType {
    /// Represents class `__new__` method
    New(PyMethodDef),
    /// Represents class `__call__` method
    Call(PyMethodDef),
    /// Represents class method
    Class(PyMethodDef),
    /// Represents static method
    Static(PyMethodDef),
    /// Represents normal method
    Method(PyMethodDef),
    /// Represents class attribute, used by `#[attribute]`
    ClassAttribute(PyClassAttributeDef),
    /// Represents getter descriptor, used by `#[getter]`
    Getter(PyGetterDef),
    /// Represents setter descriptor, used by `#[setter]`
    Setter(PySetterDef),
}

#[derive(Copy, Clone, Debug)]
pub enum PyMethodType {
    PyCFunction(ffi::PyCFunction),
    PyCFunctionWithKeywords(ffi::PyCFunctionWithKeywords),
    PyNewFunc(ffi::newfunc),
    PyInitFunc(ffi::initproc),
}

#[derive(Copy, Clone, Debug)]
pub struct PyMethodDef {
    pub ml_name: &'static str,
    pub ml_meth: PyMethodType,
    pub ml_flags: c_int,
    pub ml_doc: &'static str,
}

#[derive(Copy, Clone)]
pub struct PyClassAttributeDef {
    pub name: &'static str,
    pub meth: for<'p> fn(Python<'p>) -> PyObject,
}

#[derive(Copy, Clone, Debug)]
pub struct PyGetterDef {
    pub name: &'static str,
    pub meth: ffi::getter,
    pub doc: &'static str,
}

#[derive(Copy, Clone, Debug)]
pub struct PySetterDef {
    pub name: &'static str,
    pub meth: ffi::setter,
    pub doc: &'static str,
}

unsafe impl Sync for PyMethodDef {}

unsafe impl Sync for ffi::PyMethodDef {}

unsafe impl Sync for PyGetterDef {}

unsafe impl Sync for PySetterDef {}

unsafe impl Sync for ffi::PyGetSetDef {}

fn get_name(name: &str) -> *const std::os::raw::c_char {
    CString::new(name)
        .expect("Method name must not contain NULL byte")
        .into_raw() as _
}

fn get_doc(doc: &'static str) -> *const std::os::raw::c_char {
    CStr::from_bytes_with_nul(doc.as_bytes())
        .expect("Document must be terminated with NULL byte")
        .as_ptr()
}

impl PyMethodDef {
    /// Convert `PyMethodDef` to Python method definition struct `ffi::PyMethodDef`
    pub fn as_method_def(&self) -> ffi::PyMethodDef {
        let meth = match self.ml_meth {
            PyMethodType::PyCFunction(meth) => meth,
            PyMethodType::PyCFunctionWithKeywords(meth) => unsafe { std::mem::transmute(meth) },
            PyMethodType::PyNewFunc(meth) => unsafe { std::mem::transmute(meth) },
            PyMethodType::PyInitFunc(meth) => unsafe { std::mem::transmute(meth) },
        };

        ffi::PyMethodDef {
            ml_name: get_name(self.ml_name),
            ml_meth: Some(meth),
            ml_flags: self.ml_flags,
            ml_doc: get_doc(self.ml_doc),
        }
    }
}

// Manual implementation because `Python<'_>` does not implement `Debug` and
// trait bounds on `fn` compiler-generated derive impls are too restrictive.
impl fmt::Debug for PyClassAttributeDef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PyClassAttributeDef")
            .field("name", &self.name)
            .finish()
    }
}

impl PyGetterDef {
    /// Copy descriptor information to `ffi::PyGetSetDef`
    pub fn copy_to(&self, dst: &mut ffi::PyGetSetDef) {
        if dst.name.is_null() {
            dst.name = get_name(self.name) as _;
        }
        if dst.doc.is_null() {
            dst.doc = get_doc(self.doc) as _;
        }
        dst.get = Some(self.meth);
    }
}

impl PySetterDef {
    /// Copy descriptor information to `ffi::PyGetSetDef`
    pub fn copy_to(&self, dst: &mut ffi::PyGetSetDef) {
        if dst.name.is_null() {
            dst.name = get_name(self.name) as _;
        }
        if dst.doc.is_null() {
            dst.doc = get_doc(self.doc) as _;
        }
        dst.set = Some(self.meth);
    }
}

/// Indicates that the type `T` has some Python methods.
pub trait PyMethods {
    /// Returns all methods that are defined for a class.
    fn py_methods() -> Vec<&'static PyMethodDefType> {
        Vec::new()
    }
}

/// Implementation detail. Only to be used through our proc macro code.
/// Method storage for `#[pyclass]`.
/// Allows arbitrary `#[pymethod]/#[pyproto]` blocks to submit their methods,
/// which are eventually collected by `#[pyclass]`.
#[doc(hidden)]
#[cfg(feature = "macros")]
pub trait PyMethodsInventory: inventory::Collect {
    /// Create a new instance
    fn new(methods: &'static [PyMethodDefType]) -> Self;

    /// Returns the methods for a single `#[pymethods] impl` block
    fn get(&self) -> &'static [PyMethodDefType];
}

/// Implemented for `#[pyclass]` in our proc macro code.
/// Indicates that the pyclass has its own method storage.
#[doc(hidden)]
#[cfg(feature = "macros")]
pub trait HasMethodsInventory {
    type Methods: PyMethodsInventory;
}

#[cfg(feature = "macros")]
impl<T: HasMethodsInventory> PyMethods for T {
    fn py_methods() -> Vec<&'static PyMethodDefType> {
        inventory::iter::<T::Methods>
            .into_iter()
            .flat_map(PyMethodsInventory::get)
            .collect()
    }
}
