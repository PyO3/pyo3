// Copyright (c) 2017-present PyO3 Project and Contributors

use crate::{ffi, PyObject, Python};
use std::ffi::CStr;
use std::fmt;
use std::os::raw::c_int;

/// `PyMethodDefType` represents different types of Python callable objects.
/// It is used by the `#[pymethods]` and `#[pyproto]` annotations.
#[derive(Debug)]
pub enum PyMethodDefType {
    /// Represents class `__new__` method
    New(PyMethodDef<ffi::newfunc>),
    /// Represents class `__call__` method
    Call(PyMethodDef<ffi::PyCFunctionWithKeywords>),
    /// Represents class method
    Class(PyMethodDef<PyMethodType>),
    /// Represents static method
    Static(PyMethodDef<PyMethodType>),
    /// Represents normal method
    Method(PyMethodDef<PyMethodType>),
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
}

#[derive(Clone, Debug)]
pub struct PyMethodDef<MethodT> {
    pub(crate) ml_name: &'static CStr,
    pub(crate) ml_meth: MethodT,
    pub(crate) ml_flags: c_int,
    pub(crate) ml_doc: &'static CStr,
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

// Safe because ml_meth (the T) cannot be accessed outside of the crate, so only safe-to-sync values
// are stored in this structure.
unsafe impl<T> Sync for PyMethodDef<T> {}

unsafe impl Sync for ffi::PyMethodDef {}

unsafe impl Sync for PyGetterDef {}

unsafe impl Sync for PySetterDef {}

unsafe impl Sync for ffi::PyGetSetDef {}

fn get_name(name: &str) -> &CStr {
    CStr::from_bytes_with_nul(name.as_bytes())
        .expect("Method name must be terminated with NULL byte")
}

fn get_doc(doc: &str) -> &CStr {
    CStr::from_bytes_with_nul(doc.as_bytes()).expect("Document must be terminated with NULL byte")
}

impl PyMethodDef<ffi::newfunc> {
    /// Define a `__new__` function.
    pub fn new_func(name: &'static str, newfunc: ffi::newfunc, doc: &'static str) -> Self {
        Self {
            ml_name: get_name(name),
            ml_meth: newfunc,
            ml_flags: ffi::METH_VARARGS | ffi::METH_KEYWORDS,
            ml_doc: get_doc(doc),
        }
    }
}

impl PyMethodDef<ffi::PyCFunctionWithKeywords> {
    /// Define a `__call__` function.
    pub fn call_func(
        name: &'static str,
        callfunc: ffi::PyCFunctionWithKeywords,
        flags: c_int,
        doc: &'static str,
    ) -> Self {
        Self {
            ml_name: get_name(name),
            ml_meth: callfunc,
            ml_flags: flags | ffi::METH_VARARGS | ffi::METH_KEYWORDS,
            ml_doc: get_doc(doc),
        }
    }
}

impl PyMethodDef<PyMethodType> {
    /// Define a function with no `*args` and `**kwargs`.
    pub fn cfunction(name: &'static str, cfunction: ffi::PyCFunction, doc: &'static str) -> Self {
        Self {
            ml_name: get_name(name),
            ml_meth: PyMethodType::PyCFunction(cfunction),
            ml_flags: ffi::METH_NOARGS,
            ml_doc: get_doc(doc),
        }
    }

    /// Define a function that can take `*args` and `**kwargs`.
    pub fn cfunction_with_keywords(
        name: &'static str,
        cfunction: ffi::PyCFunctionWithKeywords,
        flags: c_int,
        doc: &'static str,
    ) -> Self {
        Self {
            ml_name: get_name(name),
            ml_meth: PyMethodType::PyCFunctionWithKeywords(cfunction),
            ml_flags: flags | ffi::METH_VARARGS | ffi::METH_KEYWORDS,
            ml_doc: get_doc(doc),
        }
    }

    /// Convert `PyMethodDef` to Python method definition struct `ffi::PyMethodDef`
    pub fn as_method_def(&self) -> ffi::PyMethodDef {
        let meth = match self.ml_meth {
            PyMethodType::PyCFunction(meth) => meth,
            PyMethodType::PyCFunctionWithKeywords(meth) => unsafe { std::mem::transmute(meth) },
        };

        ffi::PyMethodDef {
            ml_name: self.ml_name.as_ptr(),
            ml_meth: Some(meth),
            ml_flags: self.ml_flags,
            ml_doc: self.ml_doc.as_ptr(),
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

#[cfg(feature = "macros")]
impl<T: HasMethodsInventory> PyMethods for T {
    fn py_methods() -> Vec<&'static PyMethodDefType> {
        inventory::iter::<T::Methods>
            .into_iter()
            .flat_map(PyMethodsInventory::get)
            .collect()
    }
}
