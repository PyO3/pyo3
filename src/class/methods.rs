// Copyright (c) 2017-present PyO3 Project and Contributors

use std;
use std::ffi::CString;

use ffi;
use err::PyResult;
use objects::PyObject;
use python::Python;

static NO_PY_METHODS: &'static [PyMethodDefType] = &[];

pub enum PyMethodDefType {
    New(PyMethodDef),
    Call(PyMethodDef),
    Class(PyMethodDef),
    Static(PyMethodDef),
    Method(PyMethodDef),
    Getter(PyGetterDef),
    Setter(PySetterDef),
}

#[derive(Copy, Clone)]
pub enum PyMethodType {
    PyCFunction(ffi::PyCFunction),
    PyCFunctionWithKeywords(ffi::PyCFunctionWithKeywords),
    PyNoArgsFunction(ffi::PyNoArgsFunction),
    PyNewFunc(ffi::newfunc),
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
        };

        ffi::PyMethodDef {
            ml_name: CString::new(self.ml_name).expect(
                "Method name must not contain NULL byte").into_raw(),
            ml_meth: Some(meth),
            ml_flags: self.ml_flags,
            ml_doc: 0 as *const ::c_char,
        }
    }

    pub fn as_method_descr(&self, py: Python, ty: *mut ffi::PyTypeObject) -> PyResult<PyObject> {
        unsafe {
            if self.ml_flags & ffi::METH_CLASS != 0 {
                PyObject::from_owned_ptr_or_err(
                    py, ffi::PyDescr_NewClassMethod(
                        ty, Box::into_raw(Box::new(self.as_method_def()))))
            }
            else if self.ml_flags & ffi::METH_STATIC != 0 {
                PyObject::from_owned_ptr_or_err(
                    py, ffi::PyCFunction_New(
                        Box::into_raw(Box::new(self.as_method_def())), std::ptr::null_mut()))
            }
            else {
                PyObject::from_owned_ptr_or_err(
                    py, ffi::PyDescr_NewMethod(
                        ty, Box::into_raw(Box::new(self.as_method_def()))))
            }
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
