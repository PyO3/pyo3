use crate::callback::IntoPyCallbackOutput;
use crate::exceptions::PyStopAsyncIteration;
use crate::gil::LockGIL;
use crate::impl_::panic::PanicTrap;
use crate::pycell::{PyBorrowError, PyBorrowMutError};
use crate::pyclass::boolean_struct::False;
use crate::types::any::PyAnyMethods;
#[cfg(feature = "gil-refs")]
use crate::types::{PyModule, PyType};
use crate::{
    ffi, Borrowed, Bound, DowncastError, Py, PyAny, PyClass, PyClassInitializer, PyErr, PyObject,
    PyRef, PyRefMut, PyResult, PyTraverseError, PyTypeCheck, PyVisit, Python,
};
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
    pub fn as_ptr(self) -> *mut ffi::PyObject {
        self.0
    }

    #[cfg(not(Py_3_8))]
    #[inline]
    pub fn as_ptr(self) -> *mut ffi::PyObject {
        // Safety: returning a borrowed pointer to Python `None` singleton
        unsafe { ffi::Py_None() }
    }
}

/// `PyMethodDefType` represents different types of Python callable objects.
/// It is used by the `#[pymethods]` attribute.
#[cfg_attr(test, derive(Clone))]
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
    /// Represents a struct member
    StructMember(ffi::PyMemberDef),
}

#[derive(Copy, Clone, Debug)]
pub enum PyMethodType {
    PyCFunction(ffi::PyCFunction),
    PyCFunctionWithKeywords(ffi::PyCFunctionWithKeywords),
    #[cfg(not(Py_LIMITED_API))]
    PyCFunctionFastWithKeywords(ffi::_PyCFunctionFastWithKeywords),
}

pub type PyClassAttributeFactory = for<'p> fn(Python<'p>) -> PyResult<PyObject>;

// TODO: it would be nice to use CStr in these types, but then the constructors can't be const fn
// until `CStr::from_bytes_with_nul_unchecked` is const fn.

#[derive(Clone, Debug)]
pub struct PyMethodDef {
    pub(crate) ml_name: &'static CStr,
    pub(crate) ml_meth: PyMethodType,
    pub(crate) ml_flags: c_int,
    pub(crate) ml_doc: &'static CStr,
}

#[derive(Copy, Clone)]
pub struct PyClassAttributeDef {
    pub(crate) name: &'static CStr,
    pub(crate) meth: PyClassAttributeFactory,
}

#[derive(Clone)]
pub struct PyGetterDef {
    pub(crate) name: &'static CStr,
    pub(crate) meth: Getter,
    pub(crate) doc: &'static CStr,
}

#[derive(Clone)]
pub struct PySetterDef {
    pub(crate) name: &'static CStr,
    pub(crate) meth: Setter,
    pub(crate) doc: &'static CStr,
}

unsafe impl Sync for PyMethodDef {}

unsafe impl Sync for PyGetterDef {}

unsafe impl Sync for PySetterDef {}

impl PyMethodDef {
    /// Define a function with no `*args` and `**kwargs`.
    pub const fn noargs(
        ml_name: &'static CStr,
        cfunction: ffi::PyCFunction,
        ml_doc: &'static CStr,
    ) -> Self {
        Self {
            ml_name,
            ml_meth: PyMethodType::PyCFunction(cfunction),
            ml_flags: ffi::METH_NOARGS,
            ml_doc,
        }
    }

    /// Define a function that can take `*args` and `**kwargs`.
    pub const fn cfunction_with_keywords(
        ml_name: &'static CStr,
        cfunction: ffi::PyCFunctionWithKeywords,
        ml_doc: &'static CStr,
    ) -> Self {
        Self {
            ml_name,
            ml_meth: PyMethodType::PyCFunctionWithKeywords(cfunction),
            ml_flags: ffi::METH_VARARGS | ffi::METH_KEYWORDS,
            ml_doc,
        }
    }

    /// Define a function that can take `*args` and `**kwargs`.
    #[cfg(not(Py_LIMITED_API))]
    pub const fn fastcall_cfunction_with_keywords(
        ml_name: &'static CStr,
        cfunction: ffi::_PyCFunctionFastWithKeywords,
        ml_doc: &'static CStr,
    ) -> Self {
        Self {
            ml_name,
            ml_meth: PyMethodType::PyCFunctionFastWithKeywords(cfunction),
            ml_flags: ffi::METH_FASTCALL | ffi::METH_KEYWORDS,
            ml_doc,
        }
    }

    pub const fn flags(mut self, flags: c_int) -> Self {
        self.ml_flags |= flags;
        self
    }

    /// Convert `PyMethodDef` to Python method definition struct `ffi::PyMethodDef`
    pub(crate) fn as_method_def(&self) -> ffi::PyMethodDef {
        let meth = match self.ml_meth {
            PyMethodType::PyCFunction(meth) => ffi::PyMethodDefPointer { PyCFunction: meth },
            PyMethodType::PyCFunctionWithKeywords(meth) => ffi::PyMethodDefPointer {
                PyCFunctionWithKeywords: meth,
            },
            #[cfg(not(Py_LIMITED_API))]
            PyMethodType::PyCFunctionFastWithKeywords(meth) => ffi::PyMethodDefPointer {
                _PyCFunctionFastWithKeywords: meth,
            },
        };

        ffi::PyMethodDef {
            ml_name: self.ml_name.as_ptr(),
            ml_meth: meth,
            ml_flags: self.ml_flags,
            ml_doc: self.ml_doc.as_ptr(),
        }
    }
}

impl PyClassAttributeDef {
    /// Define a class attribute.
    pub const fn new(name: &'static CStr, meth: PyClassAttributeFactory) -> Self {
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
    pub const fn new(name: &'static CStr, getter: Getter, doc: &'static CStr) -> Self {
        Self {
            name,
            meth: getter,
            doc,
        }
    }
}

impl PySetterDef {
    /// Define a setter.
    pub const fn new(name: &'static CStr, setter: Setter, doc: &'static CStr) -> Self {
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
    let slf = Borrowed::from_ptr_unchecked(py, slf).downcast_unchecked::<T>();
    let borrow = PyRef::try_borrow_threadsafe(&slf);
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
#[cfg(feature = "gil-refs")]
impl<'a> From<BoundRef<'a, 'a, PyType>> for &'a PyType {
    #[inline]
    fn from(bound: BoundRef<'a, 'a, PyType>) -> Self {
        bound.0.as_gil_ref()
    }
}

#[cfg(feature = "gil-refs")]
impl<'a> From<BoundRef<'a, 'a, PyModule>> for &'a PyModule {
    #[inline]
    fn from(bound: BoundRef<'a, 'a, PyModule>) -> Self {
        bound.0.as_gil_ref()
    }
}

#[allow(deprecated)]
#[cfg(feature = "gil-refs")]
impl<'a, 'py, T: PyClass> From<BoundRef<'a, 'py, T>> for &'a crate::PyCell<T> {
    #[inline]
    fn from(bound: BoundRef<'a, 'py, T>) -> Self {
        bound.0.as_gil_ref()
    }
}

impl<'a, 'py, T: PyClass> TryFrom<BoundRef<'a, 'py, T>> for PyRef<'py, T> {
    type Error = PyBorrowError;
    #[inline]
    fn try_from(value: BoundRef<'a, 'py, T>) -> Result<Self, Self::Error> {
        value.0.try_borrow()
    }
}

impl<'a, 'py, T: PyClass<Frozen = False>> TryFrom<BoundRef<'a, 'py, T>> for PyRefMut<'py, T> {
    type Error = PyBorrowMutError;
    #[inline]
    fn try_from(value: BoundRef<'a, 'py, T>) -> Result<Self, Self::Error> {
        value.0.try_borrow_mut()
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

impl<'py, T> std::ops::Deref for BoundRef<'_, 'py, T> {
    type Target = Bound<'py, T>;
    #[inline]
    fn deref(&self) -> &Self::Target {
        self.0
    }
}

pub unsafe fn tp_new_impl<T: PyClass>(
    py: Python<'_>,
    initializer: PyClassInitializer<T>,
    target_type: *mut ffi::PyTypeObject,
) -> PyResult<*mut ffi::PyObject> {
    initializer
        .create_class_object_of_type(py, target_type)
        .map(Bound::into_ptr)
}
