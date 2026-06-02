// TODO https://github.com/PyO3/pyo3/issues/5487
#![allow(clippy::undocumented_unsafe_blocks)]

//! Contains initialization utilities for `#[pyclass]`.
use crate::exceptions::PyTypeError;
use crate::ffi_ptr_ext::FfiPtrExt;
use crate::impl_::pyclass::PyClassBaseType;
use crate::internal::get_slot::TP_NEW;
use crate::types::{PyTuple, PyType, PyTypeMethods};
use crate::{
    ffi, IntoPyObject, IntoPyObjectExt, PyClass, PyClassInitializer, PyErr, PyResult, Python,
};
use crate::{ffi::PyTypeObject, type_object::PyTypeInfo};
use core::marker::PhantomData;

/// Initializer for Python types.
///
/// This trait is used internally for distinguishing `#[pyclass]` and
/// Python native types.
pub(crate) trait PyObjectInit<T>: Sized {
    /// # Safety
    /// - `subtype` must be a valid pointer to a type object of T or a subclass.
    unsafe fn into_new_object(
        self,
        py: Python<'_>,
        subtype: *mut PyTypeObject,
    ) -> PyResult<*mut ffi::PyObject>;
}

/// Initializer for Python native types, like `PyDict`.
pub struct PyNativeTypeInitializer<T: PyTypeInfo>(pub PhantomData<T>);

impl<T: PyTypeInfo> PyObjectInit<T> for PyNativeTypeInitializer<T> {
    unsafe fn into_new_object(
        self,
        py: Python<'_>,
        subtype: *mut PyTypeObject,
    ) -> PyResult<*mut ffi::PyObject> {
        unsafe fn inner(
            py: Python<'_>,
            type_ptr: *mut PyTypeObject,
            subtype: *mut PyTypeObject,
        ) -> PyResult<*mut ffi::PyObject> {
            let tp_new = unsafe {
                type_ptr
                    .cast::<ffi::PyObject>()
                    .assume_borrowed_unchecked(py)
                    .cast_unchecked::<PyType>()
                    .get_slot(TP_NEW)
                    .ok_or_else(|| PyTypeError::new_err("base type without tp_new"))?
            };

            // TODO: make it possible to provide real arguments to the base tp_new
            let obj =
                unsafe { tp_new(subtype, PyTuple::empty(py).as_ptr(), core::ptr::null_mut()) };
            if obj.is_null() {
                Err(PyErr::fetch(py))
            } else {
                Ok(obj)
            }
        }
        unsafe { inner(py, T::type_object_raw(py), subtype) }
    }
}

pub struct TpNewValueTypeResolver<ClassT, ValueT>(
    ResolveToArbitraryObject,
    PhantomData<(ClassT, ValueT)>,
);
pub struct ResolveToArbitraryObject(());

/// First step of machinery for resolving the type of `#[new]` return values, which is used to
/// implement specialization for the various cases.
///
/// Call `.resolve()` on the returned tag to get the final tag type.
///
/// This resolution step is necessary in order to encode the preference to go via `PyClassInitializer<T>`
/// for new instances of `ClassT`. Without this step, the fallback to `IntoPyObject` conflicts for
/// `ClassT` because that implementation ignores the `cls` parameter for `PyClassInit` (and would
/// therefore be incorrect when instantiating subclasses).
pub fn tp_new_resolver<ClassT, ValueT>(_: &ValueT) -> TpNewValueTypeResolver<ClassT, ValueT> {
    TpNewValueTypeResolver(ResolveToArbitraryObject(()), PhantomData)
}

impl<ClassT, ValueT> core::ops::Deref for TpNewValueTypeResolver<ClassT, ValueT> {
    type Target = ResolveToArbitraryObject;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// Second stage of machinery for resolving the type of `#[new]` return values. This
/// is the specialized implementation for the case where the return type should be
/// used to build a new class object (`T`, `PyClassInitializer<T>`, `(S, B)` tuples).
#[expect(private_bounds, reason = "internal trait implementation")]
impl<ClassT, ValueT> TpNewValueTypeResolver<ClassT, ValueT>
where
    ClassT: PyClass,
    ValueT: IntoPyClassInitializer<ClassT>,
{
    pub fn resolve(&self, value: ValueT) -> PyClassInitializer<ClassT> {
        value.into_pyclass_initializer()
    }
}

/// All other conversions fall back to IntoPyObject via the deref implementation
impl ResolveToArbitraryObject {
    pub fn resolve<ValueT>(&self, value: ValueT) -> ValueT {
        value
    }
}

#[diagnostic::on_unimplemented(
    message = "`{Self}` cannot be used as the return value for `#[new]` methods",
    note = "all types which implement `IntoPyObject` are suitable return values for `#[new]` methods",
    note = "`PyClassInitializer<T: PyClass>` may also be used and is necessary for `#[pyclass(extends = '...')]` types",
    label = "must be a type which implements `IntoPyObject`, a `PyClassInitializer<{Self}>`, or a `Result<T>` wrapping such types"
)]
pub(crate) trait PyClassInit<'py, T> {
    /// # Safety
    /// - `cls` must be the type object for `T` (or a subclass)
    unsafe fn init(
        self,
        cls: crate::Borrowed<'_, 'py, crate::types::PyType>,
    ) -> PyResult<crate::Bound<'py, crate::PyAny>>;
}

impl<'py, ClassT, T> PyClassInit<'py, ClassT> for T
where
    T: IntoPyObject<'py>,
{
    unsafe fn init(
        self,
        cls: crate::Borrowed<'_, 'py, crate::types::PyType>,
    ) -> PyResult<crate::Bound<'py, crate::PyAny>> {
        self.into_bound_py_any(cls.py())
    }
}

impl<'py, T> PyClassInit<'py, T> for PyClassInitializer<T>
where
    T: PyClass,
{
    unsafe fn init(
        self,
        cls: crate::Borrowed<'_, 'py, crate::types::PyType>,
    ) -> PyResult<crate::Bound<'py, crate::PyAny>> {
        // SAFETY: caller has guaranteed that `cls` is correct object
        unsafe { self.create_class_object_of_type(cls.py(), cls.as_type_ptr()) }
            .map(crate::Bound::into_any)
    }
}

/// Analagous to `Into<PyClassInitializer<T>>`, but just an internal equivalent
/// to avoid allowing user code to define custom return types from `#[new]`.
trait IntoPyClassInitializer<T: PyClass>: Sized {
    fn into_pyclass_initializer(self) -> PyClassInitializer<T>;
}

impl<T: PyClass> IntoPyClassInitializer<T> for PyClassInitializer<T> {
    fn into_pyclass_initializer(self) -> PyClassInitializer<T> {
        self
    }
}

impl<T> IntoPyClassInitializer<T> for T
where
    T: PyClass,
    T::BaseType: PyClassBaseType<Initializer = PyNativeTypeInitializer<T::BaseType>>,
{
    fn into_pyclass_initializer(self) -> PyClassInitializer<T> {
        self.into()
    }
}

impl<S, B> IntoPyClassInitializer<S> for (S, B)
where
    S: PyClass<BaseType = B>,
    B: PyClass + PyClassBaseType<Initializer = PyClassInitializer<B>>,
    B::BaseType: PyClassBaseType<Initializer = PyNativeTypeInitializer<B::BaseType>>,
{
    fn into_pyclass_initializer(self) -> PyClassInitializer<S> {
        self.into()
    }
}
