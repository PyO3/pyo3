// Copyright (c) 2017-present PyO3 Project and Contributors

use std;
use std::ffi::CString;

use ffi;

static NO_PY_METHODS: &'static [PyMethodDefType] = &[];

/// `PyMethodDefType` represents different types of python callable objects.
/// It is used by `#[py::methods]` and `#[py::proto]` annotations.
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
    pub ml_flags: ::c_int,
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
            PyMethodType::PyCFunctionWithKeywords(meth) =>
                unsafe {
                    std::mem::transmute::<ffi::PyCFunctionWithKeywords, ffi::PyCFunction>(meth)
                },
            PyMethodType::PyNoArgsFunction(meth) =>
                unsafe {
                    std::mem::transmute::<ffi::PyNoArgsFunction, ffi::PyCFunction>(meth)
                },
            PyMethodType::PyNewFunc(meth) =>
                unsafe {
                    std::mem::transmute::<ffi::newfunc, ffi::PyCFunction>(meth)
                },
            PyMethodType::PyInitFunc(meth) =>
                unsafe {
                    std::mem::transmute::<ffi::initproc, ffi::PyCFunction>(meth)
                },
        };

        ffi::PyMethodDef {
            ml_name: CString::new(self.ml_name).expect(
                "Method name must not contain NULL byte").into_raw(),
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
            dst.name = CString::new(self.name).expect(
                "Method name must not contain NULL byte").into_raw();
        }
        dst.get = Some(self.meth);
    }
}

impl PySetterDef {
    /// Copy descriptor information to `ffi::PyGetSetDef`
    pub fn copy_to(&self, dst: &mut ffi::PyGetSetDef) {
        if dst.name.is_null() {
            dst.name = CString::new(self.name).expect(
                "Method name must not contain NULL byte").into_raw();
        }
        dst.set = Some(self.meth);
    }
}

#[doc(hidden)]
pub trait PyMethodsProtocolImpl {
    fn py_methods() -> &'static [PyMethodDefType];
}

impl<T> PyMethodsProtocolImpl for T {
    default fn py_methods() -> &'static [PyMethodDefType] {
        NO_PY_METHODS
    }
}

#[doc(hidden)]
pub trait PyPropMethodsProtocolImpl {
    fn py_methods() -> &'static [PyMethodDefType];
}

impl<T> PyPropMethodsProtocolImpl for T {
    default fn py_methods() -> &'static [PyMethodDefType] {
        NO_PY_METHODS
    }
}
