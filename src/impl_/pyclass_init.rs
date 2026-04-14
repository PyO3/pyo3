//! Contains initialization utilities for `#[pyclass]`.
use crate::exceptions::PyTypeError;
use crate::ffi_ptr_ext::FfiPtrExt;
use crate::impl_::pyclass::PyClassBaseType;
use crate::internal::get_slot::TP_NEW;
use crate::types::{PyTuple, PyType};
use crate::{ffi, PyClass, PyClassInitializer, PyErr, PyResult, Python};
use crate::{ffi::PyTypeObject, sealed::Sealed, type_object::PyTypeInfo};
use std::cell::Cell;
use std::marker::PhantomData;

thread_local! {
    static NATIVE_TYPE_CTOR_ARGS: Cell<(*mut ffi::PyObject, *mut ffi::PyObject)> =
        const { Cell::new((std::ptr::null_mut(), std::ptr::null_mut())) };
}

pub struct NativeTypeConstructorArgsGuard {
    prev: (*mut ffi::PyObject, *mut ffi::PyObject),
}

impl NativeTypeConstructorArgsGuard {
    pub fn push(args: *mut ffi::PyObject, kwargs: *mut ffi::PyObject) -> Self {
        let prev = NATIVE_TYPE_CTOR_ARGS.with(|cell| {
            let prev = cell.get();
            cell.set((args, kwargs));
            prev
        });
        Self { prev }
    }

    fn current() -> (*mut ffi::PyObject, *mut ffi::PyObject) {
        NATIVE_TYPE_CTOR_ARGS.with(Cell::get)
    }
}

impl Drop for NativeTypeConstructorArgsGuard {
    fn drop(&mut self) {
        NATIVE_TYPE_CTOR_ARGS.with(|cell| cell.set(self.prev));
    }
}

/// Initializer for Python types.
///
/// This trait is intended to use internally for distinguishing `#[pyclass]` and
/// Python native types.
pub trait PyObjectInit<T>: Sized + Sealed {
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
            ctor_args: (*mut ffi::PyObject, *mut ffi::PyObject),
        ) -> PyResult<*mut ffi::PyObject> {
            let tp_new = unsafe {
                type_ptr
                    .cast::<ffi::PyObject>()
                    .assume_borrowed_unchecked(py)
                    .cast_unchecked::<PyType>()
                    .get_slot(TP_NEW)
                    .ok_or_else(|| PyTypeError::new_err("base type without tp_new"))?
            };

            let (mut args, mut kwargs) = ctor_args;
            if std::ptr::eq(type_ptr, crate::PyAny::type_object_raw(py)) {
                args = std::ptr::null_mut();
                kwargs = std::ptr::null_mut();
            }
            let empty_args;
            let args = if args.is_null() {
                empty_args = PyTuple::empty(py);
                empty_args.as_ptr()
            } else {
                args
            };
            let obj = unsafe { tp_new(subtype, args, kwargs) };
            if obj.is_null() {
                Err(PyErr::fetch(py))
            } else {
                Ok(obj)
            }
        }
        unsafe {
            inner(
                py,
                T::type_object_raw(py),
                subtype,
                NativeTypeConstructorArgsGuard::current(),
            )
        }
    }
}

pub trait PyClassInit<'py, const IS_PYCLASS: bool, const IS_INITIALIZER_TUPLE: bool>:
    seal_pyclass_init::Sealed<'py, IS_PYCLASS, IS_INITIALIZER_TUPLE>
{
    fn init(
        self,
        cls: crate::Borrowed<'_, 'py, crate::types::PyType>,
    ) -> PyResult<crate::Bound<'py, crate::PyAny>>;
}

mod seal_pyclass_init {
    use crate::impl_::pyclass::{self, PyClassBaseType};
    use crate::impl_::pyclass_init::{PyClassInit, PyNativeTypeInitializer};
    use crate::{PyClass, PyClassInitializer, PyErr};

    pub trait Sealed<'py, const IS_PYCLASS: bool, const IS_INITIALIZER_TUPLE: bool> {}

    impl<'py, T> Sealed<'py, false, false> for T where T: crate::IntoPyObject<'py> {}
    impl<'py, T> Sealed<'py, true, false> for T
    where
        T: crate::PyClass,
        T::BaseType: pyclass::PyClassBaseType<Initializer = PyNativeTypeInitializer<T::BaseType>>,
    {
    }
    impl<'py, T, E, const IS_PYCLASS: bool, const IS_INITIALIZER_TUPLE: bool>
        Sealed<'py, IS_PYCLASS, IS_INITIALIZER_TUPLE> for Result<T, E>
    where
        T: PyClassInit<'py, IS_PYCLASS, IS_INITIALIZER_TUPLE>,
        E: Into<PyErr>,
    {
    }
    impl<'py, T> Sealed<'py, false, false> for PyClassInitializer<T> where T: PyClass {}
    impl<'py, S, B> Sealed<'py, false, true> for (S, B)
    where
        S: PyClass<BaseType = B>,
        B: PyClass + PyClassBaseType<Initializer = PyClassInitializer<B>>,
        B::BaseType: PyClassBaseType<Initializer = PyNativeTypeInitializer<B::BaseType>>,
    {
    }
}

impl<'py, T> PyClassInit<'py, false, false> for T
where
    T: crate::IntoPyObject<'py>,
{
    fn init(
        self,
        cls: crate::Borrowed<'_, 'py, crate::types::PyType>,
    ) -> PyResult<crate::Bound<'py, crate::PyAny>> {
        self.into_pyobject(cls.py())
            .map(crate::BoundObject::into_any)
            .map(crate::BoundObject::into_bound)
            .map_err(Into::into)
    }
}

impl<'py, T> PyClassInit<'py, true, false> for T
where
    T: crate::PyClass,
    T::BaseType:
        super::pyclass::PyClassBaseType<Initializer = PyNativeTypeInitializer<T::BaseType>>,
{
    fn init(
        self,
        cls: crate::Borrowed<'_, 'py, crate::types::PyType>,
    ) -> PyResult<crate::Bound<'py, crate::PyAny>> {
        PyClassInitializer::from(self).init(cls)
    }
}

impl<'py, T, E, const IS_PYCLASS: bool, const IS_INITIALIZER_TUPLE: bool>
    PyClassInit<'py, IS_PYCLASS, IS_INITIALIZER_TUPLE> for Result<T, E>
where
    T: PyClassInit<'py, IS_PYCLASS, IS_INITIALIZER_TUPLE>,
    E: Into<PyErr>,
{
    fn init(
        self,
        cls: crate::Borrowed<'_, 'py, crate::types::PyType>,
    ) -> PyResult<crate::Bound<'py, crate::PyAny>> {
        self.map_err(Into::into)?.init(cls)
    }
}

impl<'py, T> PyClassInit<'py, false, false> for PyClassInitializer<T>
where
    T: PyClass,
{
    fn init(
        self,
        cls: crate::Borrowed<'_, 'py, crate::types::PyType>,
    ) -> PyResult<crate::Bound<'py, crate::PyAny>> {
        unsafe {
            self.create_class_object_of_type(cls.py(), cls.as_ptr().cast())
                .map(crate::Bound::into_any)
        }
    }
}

impl<'py, S, B> PyClassInit<'py, false, true> for (S, B)
where
    S: PyClass<BaseType = B>,
    B: PyClass + PyClassBaseType<Initializer = PyClassInitializer<B>>,
    B::BaseType: PyClassBaseType<Initializer = PyNativeTypeInitializer<B::BaseType>>,
{
    fn init(
        self,
        cls: crate::Borrowed<'_, 'py, crate::types::PyType>,
    ) -> PyResult<crate::Bound<'py, crate::PyAny>> {
        let (sub, base) = self;
        PyClassInitializer::from(base).add_subclass(sub).init(cls)
    }
}
