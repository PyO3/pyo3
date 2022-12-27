use crate::exceptions::PyValueError;
use crate::{ffi, IntoPy, Py, PyAny, PyErr, PyObject, PyResult, PyTraverseError, Python};
use std::borrow::Cow;
use std::ffi::{CStr, CString};
use std::fmt;
use std::os::raw::c_int;

/// Python 3.8 and up - __ipow__ has modulo argument correctly populated.
#[cfg(Py_3_8)]
#[repr(transparent)]
pub struct IPowModulo(*mut ffi::PyObject);

/// Python 3.7 and older - __ipow__ does not have modulo argument correctly populated.
#[cfg(not(Py_3_8))]
#[repr(transparent)]
pub struct IPowModulo(std::mem::MaybeUninit<*mut ffi::PyObject>);

/// Helper to use as pymethod ffi definition
#[allow(non_camel_case_types)]
pub type ipowfunc = unsafe extern "C" fn(
    arg1: *mut ffi::PyObject,
    arg2: *mut ffi::PyObject,
    arg3: IPowModulo,
) -> *mut ffi::PyObject;

impl IPowModulo {
    #[cfg(Py_3_8)]
    #[inline]
    pub fn to_borrowed_any(self, py: Python<'_>) -> &PyAny {
        unsafe { py.from_borrowed_ptr::<PyAny>(self.0) }
    }

    #[cfg(not(Py_3_8))]
    #[inline]
    pub fn to_borrowed_any(self, py: Python<'_>) -> &PyAny {
        unsafe { py.from_borrowed_ptr::<PyAny>(ffi::Py_None()) }
    }
}

/// `PyMethodDefType` represents different types of Python callable objects.
/// It is used by the `#[pymethods]` attribute.
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
    #[cfg(not(Py_LIMITED_API))]
    PyCFunctionFastWithKeywords(PyCFunctionFastWithKeywords),
}

// These newtype structs serve no purpose other than wrapping which are function pointers - because
// function pointers aren't allowed in const fn, but types wrapping them are!
#[derive(Clone, Copy, Debug)]
pub struct PyCFunction(pub ffi::PyCFunction);
#[derive(Clone, Copy, Debug)]
pub struct PyCFunctionWithKeywords(pub ffi::PyCFunctionWithKeywords);
#[cfg(not(Py_LIMITED_API))]
#[derive(Clone, Copy, Debug)]
pub struct PyCFunctionFastWithKeywords(pub ffi::_PyCFunctionFastWithKeywords);
#[derive(Clone, Copy, Debug)]
pub struct PyGetter(pub ffi::getter);
#[derive(Clone, Copy, Debug)]
pub struct PySetter(pub ffi::setter);
#[derive(Clone, Copy)]
pub struct PyClassAttributeFactory(pub for<'p> fn(Python<'p>) -> PyResult<PyObject>);

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
    pub(crate) name: &'static str,
    pub(crate) meth: PyClassAttributeFactory,
}

impl PyClassAttributeDef {
    pub(crate) fn attribute_c_string(&self) -> PyResult<Cow<'static, CStr>> {
        extract_c_string(self.name, "class attribute name cannot contain nul bytes")
    }
}

#[derive(Clone, Debug)]
pub struct PyGetterDef {
    pub(crate) name: &'static str,
    pub(crate) meth: PyGetter,
    doc: &'static str,
}

#[derive(Clone, Debug)]
pub struct PySetterDef {
    pub(crate) name: &'static str,
    pub(crate) meth: PySetter,
    doc: &'static str,
}

unsafe impl Sync for PyMethodDef {}

unsafe impl Sync for PyGetterDef {}

unsafe impl Sync for PySetterDef {}

impl PyMethodDef {
    /// Define a function with no `*args` and `**kwargs`.
    pub const fn noargs(name: &'static str, cfunction: PyCFunction, doc: &'static str) -> Self {
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
        doc: &'static str,
    ) -> Self {
        Self {
            ml_name: name,
            ml_meth: PyMethodType::PyCFunctionWithKeywords(cfunction),
            ml_flags: ffi::METH_VARARGS | ffi::METH_KEYWORDS,
            ml_doc: doc,
        }
    }

    /// Define a function that can take `*args` and `**kwargs`.
    #[cfg(not(Py_LIMITED_API))]
    pub const fn fastcall_cfunction_with_keywords(
        name: &'static str,
        cfunction: PyCFunctionFastWithKeywords,
        doc: &'static str,
    ) -> Self {
        Self {
            ml_name: name,
            ml_meth: PyMethodType::PyCFunctionFastWithKeywords(cfunction),
            ml_flags: ffi::METH_FASTCALL | ffi::METH_KEYWORDS,
            ml_doc: doc,
        }
    }

    pub const fn flags(mut self, flags: c_int) -> Self {
        self.ml_flags |= flags;
        self
    }

    /// Convert `PyMethodDef` to Python method definition struct `ffi::PyMethodDef`
    pub(crate) fn as_method_def(&self) -> PyResult<(ffi::PyMethodDef, PyMethodDefDestructor)> {
        let meth = match self.ml_meth {
            PyMethodType::PyCFunction(meth) => ffi::PyMethodDefPointer {
                PyCFunction: meth.0,
            },
            PyMethodType::PyCFunctionWithKeywords(meth) => ffi::PyMethodDefPointer {
                PyCFunctionWithKeywords: meth.0,
            },
            #[cfg(not(Py_LIMITED_API))]
            PyMethodType::PyCFunctionFastWithKeywords(meth) => ffi::PyMethodDefPointer {
                _PyCFunctionFastWithKeywords: meth.0,
            },
        };

        let name = get_name(self.ml_name)?;
        let doc = get_doc(self.ml_doc)?;
        let def = ffi::PyMethodDef {
            ml_name: name.as_ptr(),
            ml_meth: meth,
            ml_flags: self.ml_flags,
            ml_doc: doc.as_ptr(),
        };
        let destructor = PyMethodDefDestructor { name, doc };
        Ok((def, destructor))
    }
}

impl PyClassAttributeDef {
    /// Define a class attribute.
    pub const fn new(name: &'static str, meth: PyClassAttributeFactory) -> Self {
        Self { name, meth }
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
    pub const fn new(name: &'static str, getter: PyGetter, doc: &'static str) -> Self {
        Self {
            name,
            meth: getter,
            doc,
        }
    }

    /// Copy descriptor information to `ffi::PyGetSetDef`
    pub fn copy_to(&self, dst: &mut ffi::PyGetSetDef) {
        if dst.name.is_null() {
            let name = get_name(self.name).unwrap();
            dst.name = name.as_ptr() as _;
            // FIXME: stop leaking name
            std::mem::forget(name);
        }
        if dst.doc.is_null() {
            let doc = get_doc(self.doc).unwrap();
            dst.doc = doc.as_ptr() as _;
            // FIXME: stop leaking doc
            std::mem::forget(doc);
        }
        dst.get = Some(self.meth.0);
    }
}

impl PySetterDef {
    /// Define a setter.
    pub const fn new(name: &'static str, setter: PySetter, doc: &'static str) -> Self {
        Self {
            name,
            meth: setter,
            doc,
        }
    }

    /// Copy descriptor information to `ffi::PyGetSetDef`
    pub fn copy_to(&self, dst: &mut ffi::PyGetSetDef) {
        if dst.name.is_null() {
            let name = get_name(self.name).unwrap();
            dst.name = name.as_ptr() as _;
            // FIXME: stop leaking name
            std::mem::forget(name);
        }
        if dst.doc.is_null() {
            let doc = get_doc(self.doc).unwrap();
            dst.doc = doc.as_ptr() as _;
            // FIXME: stop leaking doc
            std::mem::forget(doc);
        }
        dst.set = Some(self.meth.0);
    }
}

fn get_name(name: &'static str) -> PyResult<Cow<'static, CStr>> {
    extract_c_string(name, "Function name cannot contain NUL byte.")
}

fn get_doc(doc: &'static str) -> PyResult<Cow<'static, CStr>> {
    extract_c_string(doc, "Document cannot contain NUL byte.")
}

/// Unwraps the result of __traverse__ for tp_traverse
#[doc(hidden)]
#[inline]
pub fn unwrap_traverse_result(result: Result<(), PyTraverseError>) -> c_int {
    match result {
        Ok(()) => 0,
        Err(PyTraverseError(value)) => value,
    }
}

pub(crate) struct PyMethodDefDestructor {
    // These members are just to avoid leaking CStrings when possible
    #[allow(dead_code)]
    name: Cow<'static, CStr>,
    #[allow(dead_code)]
    doc: Cow<'static, CStr>,
}

// The macros need to Ok-wrap the output of user defined functions; i.e. if they're not a result, make them into one.
pub trait OkWrap<T> {
    type Error;
    fn wrap(self, py: Python<'_>) -> Result<Py<PyAny>, Self::Error>;
}

impl<T> OkWrap<T> for T
where
    T: IntoPy<PyObject>,
{
    type Error = PyErr;
    fn wrap(self, py: Python<'_>) -> PyResult<Py<PyAny>> {
        Ok(self.into_py(py))
    }
}

impl<T, E> OkWrap<T> for Result<T, E>
where
    T: IntoPy<PyObject>,
{
    type Error = E;
    fn wrap(self, py: Python<'_>) -> Result<Py<PyAny>, Self::Error> {
        self.map(|o| o.into_py(py))
    }
}

fn extract_c_string(src: &'static str, err_msg: &'static str) -> PyResult<Cow<'static, CStr>> {
    let bytes = src.as_bytes();
    let cow = match bytes {
        [] => {
            // Empty string, we can trivially refer to a static "\0" string
            Cow::Borrowed(unsafe { CStr::from_bytes_with_nul_unchecked(b"\0") })
        }
        [.., 0] => {
            // Last byte is a nul; try to create as a CStr
            let c_str =
                CStr::from_bytes_with_nul(bytes).map_err(|_| PyValueError::new_err(err_msg))?;
            Cow::Borrowed(c_str)
        }
        _ => {
            // Allocate a new CString for this
            let c_string = CString::new(bytes).map_err(|_| PyValueError::new_err(err_msg))?;
            Cow::Owned(c_string)
        }
    };
    Ok(cow)
}
