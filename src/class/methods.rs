// Copyright (c) 2017-present PyO3 Project and Contributors

use crate::{ffi, PyObject, Python};
use std::ffi::CStr;
use std::fmt;
use std::os::raw::c_int;

/// `PyMethodDefType` represents different types of Python callable objects.
/// It is used by the `#[pymethods]` and `#[pyproto]` annotations.
#[derive(Debug)]
pub enum PyMethodDefType {
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
    PyCFunction(PyCFunction),
    PyCFunctionWithKeywords(PyCFunctionWithKeywords),
}

// These two newtype structs serve no purpose other than wrapping the raw ffi types (which are
// function pointers) - because function pointers aren't allowed in const fn, but types wrapping
// them are!
#[derive(Clone, Copy, Debug)]
pub struct PyCFunction(pub ffi::PyCFunction);
#[derive(Clone, Copy, Debug)]
pub struct PyCFunctionWithKeywords(pub ffi::PyCFunctionWithKeywords);

// TODO: it would be nice to use CStr in these types, but then the constructors can't be const fn
// until `CStr::from_bytes_with_nul_unchecked` is const fn.
#[derive(Clone, Debug)]
pub struct PyMethodDef {
    pub(crate) ml_name: &'static str,
    pub(crate) ml_meth: PyMethodType,
    pub(crate) ml_flags: c_int,
    pub(crate) ml_doc: &'static str,
}

#[derive(Copy, Clone)]
pub struct PyClassAttributeDef {
    pub(crate) name: &'static CStr,
    pub(crate) meth: for<'p> fn(Python<'p>) -> PyObject,
}

#[derive(Clone, Debug)]
pub struct PyGetterDef {
    pub(crate) name: &'static CStr,
    pub(crate) meth: ffi::getter,
    doc: &'static CStr,
}

#[derive(Clone, Debug)]
pub struct PySetterDef {
    pub(crate) name: &'static CStr,
    pub(crate) meth: ffi::setter,
    doc: &'static CStr,
}

unsafe impl Sync for PyMethodDef {}

unsafe impl Sync for PyGetterDef {}

unsafe impl Sync for PySetterDef {}

fn get_name(name: &str) -> &CStr {
    CStr::from_bytes_with_nul(name.as_bytes())
        .expect("Method name must be terminated with NULL byte")
}

fn get_doc(doc: &str) -> &CStr {
    CStr::from_bytes_with_nul(doc.as_bytes()).expect("Document must be terminated with NULL byte")
}

impl PyMethodDef {
    /// Define a function with no `*args` and `**kwargs`.
    pub const fn cfunction(name: &'static str, cfunction: PyCFunction, doc: &'static str) -> Self {
        Self {
            ml_name: name,
            ml_meth: PyMethodType::PyCFunction(cfunction),
            ml_flags: ffi::METH_NOARGS,
            ml_doc: doc,
        }
    }

    /// Define a function that can take `*args` and `**kwargs`.
    pub const fn cfunction_with_keywords(
        name: &'static str,
        cfunction: PyCFunctionWithKeywords,
        flags: c_int,
        doc: &'static str,
    ) -> Self {
        Self {
            ml_name: name,
            ml_meth: PyMethodType::PyCFunctionWithKeywords(cfunction),
            ml_flags: flags | ffi::METH_VARARGS | ffi::METH_KEYWORDS,
            ml_doc: doc,
        }
    }

    /// Convert `PyMethodDef` to Python method definition struct `ffi::PyMethodDef`
    pub fn as_method_def(&self) -> ffi::PyMethodDef {
        let meth = match self.ml_meth {
            PyMethodType::PyCFunction(meth) => meth.0,
            PyMethodType::PyCFunctionWithKeywords(meth) => unsafe { std::mem::transmute(meth.0) },
        };

        ffi::PyMethodDef {
            ml_name: get_name(self.ml_name).as_ptr(),
            ml_meth: Some(meth),
            ml_flags: self.ml_flags,
            ml_doc: get_doc(self.ml_doc).as_ptr(),
        }
    }
}

impl PyClassAttributeDef {
    /// Define a class attribute.
    pub fn new(name: &'static str, meth: for<'p> fn(Python<'p>) -> PyObject) -> Self {
        Self {
            name: get_name(name),
            meth,
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
    /// Define a getter.
    pub fn new(name: &'static str, getter: ffi::getter, doc: &'static str) -> Self {
        Self {
            name: get_name(name),
            meth: getter,
            doc: get_doc(doc),
        }
    }

    /// Copy descriptor information to `ffi::PyGetSetDef`
    pub fn copy_to(&self, dst: &mut ffi::PyGetSetDef) {
        if dst.name.is_null() {
            dst.name = self.name.as_ptr() as _;
        }
        if dst.doc.is_null() {
            dst.doc = self.doc.as_ptr() as _;
        }
        dst.get = Some(self.meth);
    }
}

impl PySetterDef {
    /// Define a setter.
    pub fn new(name: &'static str, setter: ffi::setter, doc: &'static str) -> Self {
        Self {
            name: get_name(name),
            meth: setter,
            doc: get_doc(doc),
        }
    }

    /// Copy descriptor information to `ffi::PyGetSetDef`
    pub fn copy_to(&self, dst: &mut ffi::PyGetSetDef) {
        if dst.name.is_null() {
            dst.name = self.name.as_ptr() as _;
        }
        if dst.doc.is_null() {
            dst.doc = self.doc.as_ptr() as _;
        }
        dst.set = Some(self.meth);
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
    fn new(methods: Vec<PyMethodDefType>) -> Self;

    /// Returns the methods for a single `#[pymethods] impl` block
    fn get(&'static self) -> &'static [PyMethodDefType];
}

/// Implemented for `#[pyclass]` in our proc macro code.
/// Indicates that the pyclass has its own method storage.
#[doc(hidden)]
#[cfg(feature = "macros")]
pub trait HasMethodsInventory {
    type Methods: PyMethodsInventory;
}
