use crate::callback::IntoPyCallbackOutput;
use crate::exceptions::PyStopAsyncIteration;
use crate::gil::LockGIL;
use crate::impl_::panic::PanicTrap;
use crate::internal_tricks::extract_c_string;
use crate::pycell::{PyBorrowError, PyBorrowMutError};
use crate::pyclass::boolean_struct::False;
use crate::types::{any::PyAnyMethods, PyModule, PyType};
use crate::{
    ffi, Bound, DowncastError, Py, PyAny, PyCell, PyClass, PyErr, PyObject, PyRef, PyRefMut,
    PyResult, PyTraverseError, PyTypeCheck, PyVisit, Python,
};
use std::borrow::Cow;
use std::ffi::CStr;
use std::fmt;
use std::os::raw::{c_int, c_void};
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::ptr::null_mut;

/// Python 3.8 and up - __ipow__ has modulo argument correctly populated.
#[cfg(Py_3_8)]
#[repr(transparent)]
pub struct IPowModulo(*mut ffi::PyObject);

/// Python 3.7 and older - __ipow__ does not have modulo argument correctly populated.
#[cfg(not(Py_3_8))]
#[repr(transparent)]
pub struct IPowModulo(#[allow(dead_code)] std::mem::MaybeUninit<*mut ffi::PyObject>);

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
#[derive(Clone, Copy)]
pub struct PyGetter(pub Getter);
#[derive(Clone, Copy)]
pub struct PySetter(pub Setter);
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

#[derive(Clone)]
pub struct PyGetterDef {
    pub(crate) name: &'static str,
    pub(crate) meth: PyGetter,
    pub(crate) doc: &'static str,
}

#[derive(Clone)]
pub struct PySetterDef {
    pub(crate) name: &'static str,
    pub(crate) meth: PySetter,
    pub(crate) doc: &'static str,
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

/// Class getter / setters
pub(crate) type Getter =
    for<'py> unsafe fn(Python<'py>, *mut ffi::PyObject) -> PyResult<*mut ffi::PyObject>;
pub(crate) type Setter =
    for<'py> unsafe fn(Python<'py>, *mut ffi::PyObject, *mut ffi::PyObject) -> PyResult<c_int>;

impl PyGetterDef {
    /// Define a getter.
    pub const fn new(name: &'static str, getter: PyGetter, doc: &'static str) -> Self {
        Self {
            name,
            meth: getter,
            doc,
        }
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
}

/// Calls an implementation of __traverse__ for tp_traverse
#[doc(hidden)]
pub unsafe fn _call_traverse<T>(
    slf: *mut ffi::PyObject,
    impl_: fn(&T, PyVisit<'_>) -> Result<(), PyTraverseError>,
    visit: ffi::visitproc,
    arg: *mut c_void,
) -> c_int
where
    T: PyClass,
{
    // It is important the implementation of `__traverse__` cannot safely access the GIL,
    // c.f. https://github.com/PyO3/pyo3/issues/3165, and hence we do not expose our GIL
    // token to the user code and lock safe methods for acquiring the GIL.
    // (This includes enforcing the `&self` method receiver as e.g. `PyRef<Self>` could
    // reconstruct a GIL token via `PyRef::py`.)
    // Since we do not create a `GILPool` at all, it is important that our usage of the GIL
    // token does not produce any owned objects thereby calling into `register_owned`.
    let trap = PanicTrap::new("uncaught panic inside __traverse__ handler");

    let py = Python::assume_gil_acquired();
    let slf = py.from_borrowed_ptr::<PyCell<T>>(slf);
    let borrow = slf.try_borrow_threadsafe();
    let visit = PyVisit::from_raw(visit, arg, py);

    let retval = if let Ok(borrow) = borrow {
        let _lock = LockGIL::during_traverse();

        match catch_unwind(AssertUnwindSafe(move || impl_(&*borrow, visit))) {
            Ok(res) => match res {
                Ok(()) => 0,
                Err(PyTraverseError(value)) => value,
            },
            Err(_err) => -1,
        }
    } else {
        0
    };
    trap.disarm();
    retval
}

pub(crate) struct PyMethodDefDestructor {
    // These members are just to avoid leaking CStrings when possible
    #[allow(dead_code)]
    name: Cow<'static, CStr>,
    #[allow(dead_code)]
    doc: Cow<'static, CStr>,
}

pub(crate) fn get_name(name: &'static str) -> PyResult<Cow<'static, CStr>> {
    extract_c_string(name, "function name cannot contain NUL byte.")
}

pub(crate) fn get_doc(doc: &'static str) -> PyResult<Cow<'static, CStr>> {
    extract_c_string(doc, "function doc cannot contain NUL byte.")
}

// Autoref-based specialization for handling `__next__` returning `Option`

pub struct IterBaseTag;

impl IterBaseTag {
    #[inline]
    pub fn convert<Value, Target>(self, py: Python<'_>, value: Value) -> PyResult<Target>
    where
        Value: IntoPyCallbackOutput<Target>,
    {
        value.convert(py)
    }
}

pub trait IterBaseKind {
    #[inline]
    fn iter_tag(&self) -> IterBaseTag {
        IterBaseTag
    }
}

impl<Value> IterBaseKind for &Value {}

pub struct IterOptionTag;

impl IterOptionTag {
    #[inline]
    pub fn convert<Value>(
        self,
        py: Python<'_>,
        value: Option<Value>,
    ) -> PyResult<*mut ffi::PyObject>
    where
        Value: IntoPyCallbackOutput<*mut ffi::PyObject>,
    {
        match value {
            Some(value) => value.convert(py),
            None => Ok(null_mut()),
        }
    }
}

pub trait IterOptionKind {
    #[inline]
    fn iter_tag(&self) -> IterOptionTag {
        IterOptionTag
    }
}

impl<Value> IterOptionKind for Option<Value> {}

pub struct IterResultOptionTag;

impl IterResultOptionTag {
    #[inline]
    pub fn convert<Value, Error>(
        self,
        py: Python<'_>,
        value: Result<Option<Value>, Error>,
    ) -> PyResult<*mut ffi::PyObject>
    where
        Value: IntoPyCallbackOutput<*mut ffi::PyObject>,
        Error: Into<PyErr>,
    {
        match value {
            Ok(Some(value)) => value.convert(py),
            Ok(None) => Ok(null_mut()),
            Err(err) => Err(err.into()),
        }
    }
}

pub trait IterResultOptionKind {
    #[inline]
    fn iter_tag(&self) -> IterResultOptionTag {
        IterResultOptionTag
    }
}

impl<Value, Error> IterResultOptionKind for Result<Option<Value>, Error> {}

// Autoref-based specialization for handling `__anext__` returning `Option`

pub struct AsyncIterBaseTag;

impl AsyncIterBaseTag {
    #[inline]
    pub fn convert<Value, Target>(self, py: Python<'_>, value: Value) -> PyResult<Target>
    where
        Value: IntoPyCallbackOutput<Target>,
    {
        value.convert(py)
    }
}

pub trait AsyncIterBaseKind {
    #[inline]
    fn async_iter_tag(&self) -> AsyncIterBaseTag {
        AsyncIterBaseTag
    }
}

impl<Value> AsyncIterBaseKind for &Value {}

pub struct AsyncIterOptionTag;

impl AsyncIterOptionTag {
    #[inline]
    pub fn convert<Value>(
        self,
        py: Python<'_>,
        value: Option<Value>,
    ) -> PyResult<*mut ffi::PyObject>
    where
        Value: IntoPyCallbackOutput<*mut ffi::PyObject>,
    {
        match value {
            Some(value) => value.convert(py),
            None => Err(PyStopAsyncIteration::new_err(())),
        }
    }
}

pub trait AsyncIterOptionKind {
    #[inline]
    fn async_iter_tag(&self) -> AsyncIterOptionTag {
        AsyncIterOptionTag
    }
}

impl<Value> AsyncIterOptionKind for Option<Value> {}

pub struct AsyncIterResultOptionTag;

impl AsyncIterResultOptionTag {
    #[inline]
    pub fn convert<Value, Error>(
        self,
        py: Python<'_>,
        value: Result<Option<Value>, Error>,
    ) -> PyResult<*mut ffi::PyObject>
    where
        Value: IntoPyCallbackOutput<*mut ffi::PyObject>,
        Error: Into<PyErr>,
    {
        match value {
            Ok(Some(value)) => value.convert(py),
            Ok(None) => Err(PyStopAsyncIteration::new_err(())),
            Err(err) => Err(err.into()),
        }
    }
}

pub trait AsyncIterResultOptionKind {
    #[inline]
    fn async_iter_tag(&self) -> AsyncIterResultOptionTag {
        AsyncIterResultOptionTag
    }
}

impl<Value, Error> AsyncIterResultOptionKind for Result<Option<Value>, Error> {}

/// Used in `#[classmethod]` to pass the class object to the method
/// and also in `#[pyfunction(pass_module)]`.
///
/// This is a wrapper to avoid implementing `From<Bound>` for GIL Refs.
///
/// Once the GIL Ref API is fully removed, it should be possible to simplify
/// this to just `&'a Bound<'py, T>` and `From` implementations.
pub struct BoundRef<'a, 'py, T>(pub &'a Bound<'py, T>);

impl<'a, 'py> BoundRef<'a, 'py, PyAny> {
    pub unsafe fn ref_from_ptr(py: Python<'py>, ptr: &'a *mut ffi::PyObject) -> Self {
        BoundRef(Bound::ref_from_ptr(py, ptr))
    }

    pub unsafe fn ref_from_ptr_or_opt(
        py: Python<'py>,
        ptr: &'a *mut ffi::PyObject,
    ) -> Option<Self> {
        Bound::ref_from_ptr_or_opt(py, ptr).as_ref().map(BoundRef)
    }

    pub fn downcast<T: PyTypeCheck>(self) -> Result<BoundRef<'a, 'py, T>, DowncastError<'a, 'py>> {
        self.0.downcast::<T>().map(BoundRef)
    }

    pub unsafe fn downcast_unchecked<T>(self) -> BoundRef<'a, 'py, T> {
        BoundRef(self.0.downcast_unchecked::<T>())
    }
}

// GIL Ref implementations for &'a T ran into trouble with orphan rules,
// so explicit implementations are used instead for the two relevant types.
impl<'a> From<BoundRef<'a, 'a, PyType>> for &'a PyType {
    #[inline]
    fn from(bound: BoundRef<'a, 'a, PyType>) -> Self {
        bound.0.as_gil_ref()
    }
}

impl<'a> From<BoundRef<'a, 'a, PyModule>> for &'a PyModule {
    #[inline]
    fn from(bound: BoundRef<'a, 'a, PyModule>) -> Self {
        bound.0.as_gil_ref()
    }
}

impl<'a, 'py, T: PyClass> From<BoundRef<'a, 'py, T>> for &'a PyCell<T> {
    #[inline]
    fn from(bound: BoundRef<'a, 'py, T>) -> Self {
        bound.0.as_gil_ref()
    }
}

impl<'a, 'py, T: PyClass> TryFrom<BoundRef<'a, 'py, T>> for PyRef<'py, T> {
    type Error = PyBorrowError;
    #[inline]
    fn try_from(value: BoundRef<'a, 'py, T>) -> Result<Self, Self::Error> {
        value.0.clone().into_gil_ref().try_into()
    }
}

impl<'a, 'py, T: PyClass<Frozen = False>> TryFrom<BoundRef<'a, 'py, T>> for PyRefMut<'py, T> {
    type Error = PyBorrowMutError;
    #[inline]
    fn try_from(value: BoundRef<'a, 'py, T>) -> Result<Self, Self::Error> {
        value.0.clone().into_gil_ref().try_into()
    }
}

impl<'a, 'py, T> From<BoundRef<'a, 'py, T>> for Bound<'py, T> {
    #[inline]
    fn from(bound: BoundRef<'a, 'py, T>) -> Self {
        bound.0.clone()
    }
}

impl<'a, 'py, T> From<BoundRef<'a, 'py, T>> for &'a Bound<'py, T> {
    #[inline]
    fn from(bound: BoundRef<'a, 'py, T>) -> Self {
        bound.0
    }
}

impl<T> From<BoundRef<'_, '_, T>> for Py<T> {
    #[inline]
    fn from(bound: BoundRef<'_, '_, T>) -> Self {
        bound.0.clone().unbind()
    }
}
