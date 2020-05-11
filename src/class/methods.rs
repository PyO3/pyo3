// Copyright (c) 2017-present PyO3 Project and Contributors

use crate::{ffi, PyObject, Python};
use libc::c_int;
use std::ffi::CString;
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
            ml_name: CString::new(self.ml_name)
                .expect("Method name must not contain NULL byte")
                .into_raw(),
            ml_meth: Some(meth),
            ml_flags: self.ml_flags,
            ml_doc: self.ml_doc.as_ptr() as *const _,
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
            dst.name = CString::new(self.name)
                .expect("Method name must not contain NULL byte")
                .into_raw();
        }
        if dst.doc.is_null() {
            dst.doc = self.doc.as_ptr() as *mut libc::c_char;
        }
        dst.get = Some(self.meth);
    }
}

impl PySetterDef {
    /// Copy descriptor information to `ffi::PyGetSetDef`
    pub fn copy_to(&self, dst: &mut ffi::PyGetSetDef) {
        if dst.name.is_null() {
            dst.name = CString::new(self.name)
                .expect("Method name must not contain NULL byte")
                .into_raw();
        }
        if dst.doc.is_null() {
            dst.doc = self.doc.as_ptr() as *mut libc::c_char;
        }
        dst.set = Some(self.meth);
    }
}

/// Implementation detail. Only to be used through the proc macros.
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

/// Implementation detail. Only to be used through the proc macros.
/// For pyclass derived structs, this trait collects method from all impl blocks using inventory.
#[doc(hidden)]
#[cfg(feature = "macros")]
pub trait PyMethodsImpl {
    /// Normal methods. Mainly defined by `#[pymethod]`.
    type Methods: PyMethodsInventory;

    /// Returns all methods that are defined for a class.
    fn py_methods() -> Vec<&'static PyMethodDefType> {
        inventory::iter::<Self::Methods>
            .into_iter()
            .flat_map(PyMethodsInventory::get)
            .collect()
    }
}

#[doc(hidden)]
#[cfg(not(feature = "macros"))]
pub trait PyMethodsImpl {
    fn py_methods() -> Vec<&'static PyMethodDefType> {
        Vec::new()
    }
}
