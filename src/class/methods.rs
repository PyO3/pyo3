// Copyright (c) 2017-present PyO3 Project and Contributors

use std::mem;
use std::ptr;
use std::ffi::CString;

use ::{ffi, exc, class, py_class, PyErr, Python, PyResult, PythonObject};
use objects::PyType;
use function::AbortOnDrop;
use class::NO_PY_METHODS;

pub enum PyMethodDefType {
    Method(PyMethodDef),
    Getter(PyGetterDef),
    Setter(PySetterDef),
}

#[derive(Copy, Clone)]
pub enum PyMethodType {
    PyCFunction(ffi::PyCFunction),
    PyCFunctionWithKeywords(ffi::PyCFunctionWithKeywords),
}

#[derive(Copy, Clone)]
pub struct PyMethodDef {
    pub ml_name: &'static str,
    pub ml_meth: PyMethodType,
    pub ml_flags: ::c_int,
    pub ml_doc: &'static str,
}

#[derive(Copy, Clone)]
pub struct PyGetterDef {
    pub name: &'static str,
    pub meth: ffi::getter,
    pub doc: &'static str,
}

#[derive(Copy, Clone)]
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

    pub fn as_method_def(&self) -> ffi::PyMethodDef {
        let meth = match self.ml_meth {
            PyMethodType::PyCFunction(meth) => meth,
            PyMethodType::PyCFunctionWithKeywords(meth) =>
                unsafe {
                    ::std::mem::transmute::<
                            ffi::PyCFunctionWithKeywords, ffi::PyCFunction>(meth)
                }
        };

        ffi::PyMethodDef {
            ml_name: CString::new(self.ml_name).expect(
                "Method name must not contain NULL byte").into_raw(),
            ml_meth: Some(meth),
            ml_flags: self.ml_flags,
            ml_doc: 0 as *const ::c_char,
        }
    }
}

impl PyGetterDef {
    pub fn copy_to(&self, dst: &mut ffi::PyGetSetDef) {
        if dst.name.is_null() {
            dst.name = CString::new(self.name).expect(
                "Method name must not contain NULL byte").into_raw();
        }
        dst.get = Some(self.meth.clone());
    }
}

impl PySetterDef {
    pub fn copy_to(&self, dst: &mut ffi::PyGetSetDef) {
        if dst.name.is_null() {
            dst.name = CString::new(self.name).expect(
                "Method name must not contain NULL byte").into_raw();
        }
        dst.set = Some(self.meth.clone());
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
