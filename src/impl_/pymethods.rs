use crate::exceptions::PyStopAsyncIteration;
use crate::impl_::callback::IntoPyCallbackOutput;
use crate::impl_::panic::PanicTrap;
use crate::impl_::pycell::{PyClassObject, PyClassObjectLayout};
use crate::internal::get_slot::{get_slot, TP_BASE, TP_CLEAR, TP_TRAVERSE};
use crate::internal::state::ForbidAttaching;
use crate::pycell::impl_::PyClassBorrowChecker as _;
use crate::pycell::{PyBorrowError, PyBorrowMutError};
use crate::pyclass::boolean_struct::False;
use crate::types::PyType;
use crate::{
    ffi, Bound, DowncastError, Py, PyAny, PyClass, PyClassGuard, PyClassGuardMut,
    PyClassInitializer, PyErr, PyRef, PyRefMut, PyResult, PyTraverseError, PyTypeCheck, PyVisit,
    Python,
};
use std::ffi::CStr;
use std::ffi::{c_int, c_void};
use std::fmt;
use std::marker::PhantomData;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::ptr::{null_mut, NonNull};

use super::trampoline;
use crate::internal_tricks::{clear_eq, traverse_eq};

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
    #[cfg(any(Py_3_10, not(Py_LIMITED_API)))]
    PyCFunctionFastWithKeywords(ffi::PyCFunctionFastWithKeywords),
}

pub type PyClassAttributeFactory = for<'p> fn(Python<'p>) -> PyResult<Py<PyAny>>;

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
    #[cfg(any(Py_3_10, not(Py_LIMITED_API)))]
    pub const fn fastcall_cfunction_with_keywords(
        ml_name: &'static CStr,
        cfunction: ffi::PyCFunctionFastWithKeywords,
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
            #[cfg(any(Py_3_10, not(Py_LIMITED_API)))]
            PyMethodType::PyCFunctionFastWithKeywords(meth) => ffi::PyMethodDefPointer {
                PyCFunctionFastWithKeywords: meth,
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
///
/// NB cannot accept `'static` visitor, this is a sanity check below:
///
/// ```rust,compile_fail
/// use pyo3::prelude::*;
/// use pyo3::pyclass::{PyTraverseError, PyVisit};
///
/// #[pyclass]
/// struct Foo;
///
/// #[pymethods]
/// impl Foo {
///     fn __traverse__(&self, _visit: PyVisit<'static>) -> Result<(), PyTraverseError> {
///         Ok(())
///     }
/// }
/// ```
///
/// Elided lifetime should compile ok:
///
/// ```rust,no_run
/// use pyo3::prelude::*;
/// use pyo3::pyclass::{PyTraverseError, PyVisit};
///
/// #[pyclass]
/// struct Foo;
///
/// #[pymethods]
/// impl Foo {
///     fn __traverse__(&self, _visit: PyVisit<'_>) -> Result<(), PyTraverseError> {
///         Ok(())
///     }
/// }
/// ```
#[doc(hidden)]
pub unsafe fn _call_traverse<T>(
    slf: *mut ffi::PyObject,
    impl_: fn(&T, PyVisit<'_>) -> Result<(), PyTraverseError>,
    visit: ffi::visitproc,
    arg: *mut c_void,
    current_traverse: ffi::traverseproc,
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
    let lock = ForbidAttaching::during_traverse();

    let super_retval = unsafe { call_super_traverse(slf, visit, arg, current_traverse) };
    if super_retval != 0 {
        return super_retval;
    }

    // SAFETY: `slf` is a valid Python object pointer to a class object of type T, and
    // traversal is running so no mutations can occur.
    let class_object: &PyClassObject<T> = unsafe { &*slf.cast() };

    let retval =
    // `#[pyclass(unsendable)]` types can only be deallocated by their own thread, so
    // do not traverse them if not on their owning thread :(
    if class_object.check_threadsafe().is_ok()
    // ... and we cannot traverse a type which might be being mutated by a Rust thread
    && class_object.borrow_checker().try_borrow().is_ok() {
        struct TraverseGuard<'a, T: PyClass>(&'a PyClassObject<T>);
        impl<T: PyClass> Drop for TraverseGuard<'_,  T> {
            fn drop(&mut self) {
                self.0.borrow_checker().release_borrow()
            }
        }

        // `.try_borrow()` above created a borrow, we need to release it when we're done
        // traversing the object. This allows us to read `instance` safely.
        let _guard = TraverseGuard(class_object);
        let instance = unsafe {&*class_object.contents.value.get()};

        let visit = PyVisit { visit, arg, _guard: PhantomData };

        match catch_unwind(AssertUnwindSafe(move || impl_(instance, visit))) {
            Ok(Ok(())) => 0,
            Ok(Err(traverse_error)) => traverse_error.into_inner(),
            Err(_err) => -1,
        }
    } else {
        0
    };

    // Drop lock before trap just in case dropping lock panics
    drop(lock);
    trap.disarm();
    retval
}

/// Call super-type traverse method, if necessary.
///
/// Adapted from <https://github.com/cython/cython/blob/7acfb375fb54a033f021b0982a3cd40c34fb22ac/Cython/Utility/ExtensionTypes.c#L386>
///
/// TODO: There are possible optimizations over looking up the base type in this way
/// - if the base type is known in this module, can potentially look it up directly in module state
///   (when we have it)
/// - if the base type is a Python builtin, can jut call the C function directly
/// - if the base type is a PyO3 type defined in the same module, can potentially do similar to
///   tp_alloc where we solve this at compile time
unsafe fn call_super_traverse(
    obj: *mut ffi::PyObject,
    visit: ffi::visitproc,
    arg: *mut c_void,
    current_traverse: ffi::traverseproc,
) -> c_int {
    // SAFETY: in this function here it's ok to work with raw type objects `ffi::Py_TYPE`
    // because the GC is running and so
    // - (a) we cannot do refcounting and
    // - (b) the type of the object cannot change.
    let mut ty = unsafe { ffi::Py_TYPE(obj) };
    let mut traverse: Option<ffi::traverseproc>;

    // First find the current type by the current_traverse function
    loop {
        traverse = unsafe { get_slot(ty, TP_TRAVERSE) };
        if traverse_eq(traverse, current_traverse) {
            break;
        }
        ty = unsafe { get_slot(ty, TP_BASE) };
        if ty.is_null() {
            // FIXME: return an error if current type not in the MRO? Should be impossible.
            return 0;
        }
    }

    // Get first base which has a different traverse function
    while traverse_eq(traverse, current_traverse) {
        ty = unsafe { get_slot(ty, TP_BASE) };
        if ty.is_null() {
            break;
        }
        traverse = unsafe { get_slot(ty, TP_TRAVERSE) };
    }

    // If we found a type with a different traverse function, call it
    if let Some(traverse) = traverse {
        return unsafe { traverse(obj, visit, arg) };
    }

    // FIXME same question as cython: what if the current type is not in the MRO?
    0
}

/// Calls an implementation of __clear__ for tp_clear
pub unsafe fn _call_clear(
    slf: *mut ffi::PyObject,
    impl_: for<'py> unsafe fn(Python<'py>, *mut ffi::PyObject) -> PyResult<()>,
    current_clear: ffi::inquiry,
) -> c_int {
    unsafe {
        trampoline::trampoline(move |py| {
            let super_retval = call_super_clear(py, slf, current_clear);
            if super_retval != 0 {
                return Err(PyErr::fetch(py));
            }
            impl_(py, slf)?;
            Ok(0)
        })
    }
}

/// Call super-type traverse method, if necessary.
///
/// Adapted from <https://github.com/cython/cython/blob/7acfb375fb54a033f021b0982a3cd40c34fb22ac/Cython/Utility/ExtensionTypes.c#L386>
///
/// TODO: There are possible optimizations over looking up the base type in this way
/// - if the base type is known in this module, can potentially look it up directly in module state
///   (when we have it)
/// - if the base type is a Python builtin, can jut call the C function directly
/// - if the base type is a PyO3 type defined in the same module, can potentially do similar to
///   tp_alloc where we solve this at compile time
unsafe fn call_super_clear(
    py: Python<'_>,
    obj: *mut ffi::PyObject,
    current_clear: ffi::inquiry,
) -> c_int {
    let mut ty = unsafe { PyType::from_borrowed_type_ptr(py, ffi::Py_TYPE(obj)) };
    let mut clear: Option<ffi::inquiry>;

    // First find the current type by the current_clear function
    loop {
        clear = ty.get_slot(TP_CLEAR);
        if clear_eq(clear, current_clear) {
            break;
        }
        let base = ty.get_slot(TP_BASE);
        if base.is_null() {
            // FIXME: return an error if current type not in the MRO? Should be impossible.
            return 0;
        }
        ty = unsafe { PyType::from_borrowed_type_ptr(py, base) };
    }

    // Get first base which has a different clear function
    while clear_eq(clear, current_clear) {
        let base = ty.get_slot(TP_BASE);
        if base.is_null() {
            break;
        }
        ty = unsafe { PyType::from_borrowed_type_ptr(py, base) };
        clear = ty.get_slot(TP_CLEAR);
    }

    // If we found a type with a different clear function, call it
    if let Some(clear) = clear {
        return unsafe { clear(obj) };
    }

    // FIXME same question as cython: what if the current type is not in the MRO?
    0
}

// Autoref-based specialization for handling `__next__` returning `Option`

pub struct IterBaseTag;

impl IterBaseTag {
    #[inline]
    pub fn convert<'py, Value, Target>(self, py: Python<'py>, value: Value) -> PyResult<Target>
    where
        Value: IntoPyCallbackOutput<'py, Target>,
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
    pub fn convert<'py, Value>(
        self,
        py: Python<'py>,
        value: Option<Value>,
    ) -> PyResult<*mut ffi::PyObject>
    where
        Value: IntoPyCallbackOutput<'py, *mut ffi::PyObject>,
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
    pub fn convert<'py, Value, Error>(
        self,
        py: Python<'py>,
        value: Result<Option<Value>, Error>,
    ) -> PyResult<*mut ffi::PyObject>
    where
        Value: IntoPyCallbackOutput<'py, *mut ffi::PyObject>,
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
    pub fn convert<'py, Value, Target>(self, py: Python<'py>, value: Value) -> PyResult<Target>
    where
        Value: IntoPyCallbackOutput<'py, Target>,
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
    pub fn convert<'py, Value>(
        self,
        py: Python<'py>,
        value: Option<Value>,
    ) -> PyResult<*mut ffi::PyObject>
    where
        Value: IntoPyCallbackOutput<'py, *mut ffi::PyObject>,
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
    pub fn convert<'py, Value, Error>(
        self,
        py: Python<'py>,
        value: Result<Option<Value>, Error>,
    ) -> PyResult<*mut ffi::PyObject>
    where
        Value: IntoPyCallbackOutput<'py, *mut ffi::PyObject>,
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
        unsafe { BoundRef(Bound::ref_from_ptr(py, ptr)) }
    }

    pub unsafe fn ref_from_ptr_or_opt(
        py: Python<'py>,
        ptr: &'a *mut ffi::PyObject,
    ) -> Option<Self> {
        unsafe { Bound::ref_from_ptr_or_opt(py, ptr).as_ref().map(BoundRef) }
    }

    pub unsafe fn ref_from_non_null(py: Python<'py>, ptr: &'a NonNull<ffi::PyObject>) -> Self {
        unsafe { Self(Bound::ref_from_non_null(py, ptr)) }
    }

    pub fn downcast<T: PyTypeCheck>(self) -> Result<BoundRef<'a, 'py, T>, DowncastError<'a, 'py>> {
        self.0.cast::<T>().map(BoundRef)
    }

    pub unsafe fn downcast_unchecked<T>(self) -> BoundRef<'a, 'py, T> {
        unsafe { BoundRef(self.0.cast_unchecked::<T>()) }
    }
}

impl<'a, 'py, T: PyClass> TryFrom<BoundRef<'a, 'py, T>> for PyClassGuard<'a, T> {
    type Error = PyBorrowError;
    #[inline]
    fn try_from(value: BoundRef<'a, 'py, T>) -> Result<Self, Self::Error> {
        PyClassGuard::try_borrow(value.0.as_unbound())
    }
}

impl<'a, 'py, T: PyClass<Frozen = False>> TryFrom<BoundRef<'a, 'py, T>> for PyClassGuardMut<'a, T> {
    type Error = PyBorrowMutError;
    #[inline]
    fn try_from(value: BoundRef<'a, 'py, T>) -> Result<Self, Self::Error> {
        PyClassGuardMut::try_borrow_mut(value.0.as_unbound())
    }
}

impl<'a, 'py, T: PyClass> TryFrom<BoundRef<'a, 'py, T>> for PyRef<'py, T> {
    type Error = PyBorrowError;
    #[inline]
    fn try_from(value: BoundRef<'a, 'py, T>) -> Result<Self, Self::Error> {
        PyRef::try_borrow(value.0)
    }
}

impl<'a, 'py, T: PyClass<Frozen = False>> TryFrom<BoundRef<'a, 'py, T>> for PyRefMut<'py, T> {
    type Error = PyBorrowMutError;
    #[inline]
    fn try_from(value: BoundRef<'a, 'py, T>) -> Result<Self, Self::Error> {
        PyRefMut::try_borrow(value.0)
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
    unsafe {
        initializer
            .create_class_object_of_type(py, target_type)
            .map(Bound::into_ptr)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    #[cfg(any(Py_3_10, not(Py_LIMITED_API)))]
    fn test_fastcall_function_with_keywords() {
        use super::PyMethodDef;
        use crate::types::{PyAnyMethods, PyCFunction};
        use crate::{ffi, Python};

        Python::attach(|py| {
            unsafe extern "C" fn accepts_no_arguments(
                _slf: *mut ffi::PyObject,
                _args: *const *mut ffi::PyObject,
                nargs: ffi::Py_ssize_t,
                kwargs: *mut ffi::PyObject,
            ) -> *mut ffi::PyObject {
                assert_eq!(nargs, 0);
                assert!(kwargs.is_null());
                unsafe { Python::assume_attached().None().into_ptr() }
            }

            let f = PyCFunction::internal_new(
                py,
                &PyMethodDef::fastcall_cfunction_with_keywords(
                    ffi::c_str!("test"),
                    accepts_no_arguments,
                    ffi::c_str!("doc"),
                ),
                None,
            )
            .unwrap();

            f.call0().unwrap();
        });
    }
}
