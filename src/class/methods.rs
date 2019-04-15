// Copyright (c) 2017-present PyO3 Project and Contributors

use crate::ffi;
use libc::c_int;
use std::ffi::CString;

/// `PyMethodDefType` represents different types of python callable objects.
/// It is used by `#[pymethods]` and `#[pyproto]` annotations.
#[derive(Debug)]
pub enum PyMethodDefType {
    /// Represents class `__new__` method
    New(PyMethodDef),
    /// Represents class `__init__` method
    Init(PyMethodDef),
    /// Represents class `__call__` method
    Call(PyMethodDef),
    /// Represents class method
    Class(PyMethodDef),
    /// Represents static method
    Static(PyMethodDef),
    /// Represents normal method
    Method(PyMethodDef),
    /// Represents getter descriptor, used by `#[getter]`
    Getter(PyGetterDef),
    /// Represents setter descriptor, used by `#[setter]`
    Setter(PySetterDef),
}

#[derive(Copy, Clone, Debug)]
pub enum PyMethodType {
    PyCFunction(ffi::PyCFunction),
    PyCFunctionWithKeywords(ffi::PyCFunctionWithKeywords),
    PyNoArgsFunction(ffi::PyNoArgsFunction),
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
            PyMethodType::PyNoArgsFunction(meth) => unsafe { std::mem::transmute(meth) },
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
        dst.set = Some(self.meth);
    }
}

#[doc(hidden)] // Only to be used through the proc macros, use PyMethodsProtocol in custom code
/// This trait is implemented for all pyclass so to implement the [PyMethodsProtocol]
/// through inventory
pub trait PyMethodsInventoryDispatch {
    /// This allows us to get the inventory type when only the pyclass is in scope
    type InventoryType: PyMethodsInventory;
}

#[doc(hidden)] // Only to be used through the proc macros, use PyMethodsProtocol in custom code
/// Allows arbitrary pymethod blocks to submit their methods, which are eventually collected by pyclass
pub trait PyMethodsInventory: inventory::Collect {
    /// Create a new instance
    fn new(methods: &'static [PyMethodDefType]) -> Self;

    /// Returns the methods for a single impl block
    fn get_methods(&self) -> &'static [PyMethodDefType];
}

/// The implementation of tis trait defines which methods a python type has.
///
/// For pyclass derived structs this is implemented by collecting all impl blocks through inventory
pub trait PyMethodsProtocol {
    /// Returns all methods that are defined for a class
    fn py_methods() -> Vec<&'static PyMethodDefType>;
}

impl<T> PyMethodsProtocol for T
where
    T: PyMethodsInventoryDispatch,
{
    fn py_methods() -> Vec<&'static PyMethodDefType> {
        inventory::iter::<T::InventoryType>
            .into_iter()
            .flat_map(PyMethodsInventory::get_methods)
            .collect()
    }
}
