//! Contains initialization utilities for `#[pyclass]`.
use crate::ffi_ptr_ext::FfiPtrExt;
use crate::impl_::callback::IntoPyCallbackOutput;
use crate::impl_::pyclass::{PyClassBaseType, PyClassDict, PyClassThreadChecker, PyClassWeakRef};
use crate::impl_::pyclass_init::{PyNativeTypeInitializer, PyObjectInit};
use crate::{ffi, Bound, Py, PyClass, PyResult, Python};
use crate::{
    ffi::PyTypeObject,
    pycell::impl_::{PyClassBorrowChecker, PyClassMutability, PyClassObjectContents},
};
use std::{
    cell::UnsafeCell,
    marker::PhantomData,
    mem::{ManuallyDrop, MaybeUninit},
};

/// Initializer for our `#[pyclass]` system.
///
/// You can use this type to initialize complicatedly nested `#[pyclass]`.
///
/// # Examples
///
/// ```
/// # use pyo3::prelude::*;
/// # use pyo3::py_run;
/// #[pyclass(subclass)]
/// struct BaseClass {
///     #[pyo3(get)]
///     basename: &'static str,
/// }
/// #[pyclass(extends=BaseClass, subclass)]
/// struct SubClass {
///     #[pyo3(get)]
///     subname: &'static str,
/// }
/// #[pyclass(extends=SubClass)]
/// struct SubSubClass {
///     #[pyo3(get)]
///     subsubname: &'static str,
/// }
///
/// #[pymethods]
/// impl SubSubClass {
///     #[new]
///     fn new() -> PyClassInitializer<Self> {
///         PyClassInitializer::from(BaseClass { basename: "base" })
///             .add_subclass(SubClass { subname: "sub" })
///             .add_subclass(SubSubClass {
///                 subsubname: "subsub",
///             })
///     }
/// }
/// Python::attach(|py| {
///     let typeobj = py.get_type::<SubSubClass>();
///     let sub_sub_class = typeobj.call((), None).unwrap();
///     py_run!(
///         py,
///         sub_sub_class,
///         r#"
///  assert sub_sub_class.basename == 'base'
///  assert sub_sub_class.subname == 'sub'
///  assert sub_sub_class.subsubname == 'subsub'"#
///     );
/// });
/// ```
pub struct PyClassInitializer<T: PyClass>(PyClassInitializerImpl<T>);

enum PyClassInitializerImpl<T: PyClass> {
    Existing(Py<T>),
    New {
        init: T,
        super_init: <T::BaseType as PyClassBaseType>::Initializer,
    },
}

impl<T: PyClass> PyClassInitializer<T> {
    /// Constructs a new initializer from value `T` and base class' initializer.
    ///
    /// It is recommended to use `add_subclass` instead of this method for most usage.
    #[track_caller]
    #[inline]
    pub fn new(init: T, super_init: <T::BaseType as PyClassBaseType>::Initializer) -> Self {
        // This is unsound; see https://github.com/PyO3/pyo3/issues/4452.
        assert!(
            super_init.can_be_subclassed(),
            "you cannot add a subclass to an existing value",
        );
        Self(PyClassInitializerImpl::New { init, super_init })
    }

    /// Constructs a new initializer from an initializer for the base class.
    ///
    /// # Examples
    /// ```
    /// use pyo3::prelude::*;
    ///
    /// #[pyclass(subclass)]
    /// struct BaseClass {
    ///     #[pyo3(get)]
    ///     value: i32,
    /// }
    ///
    /// impl BaseClass {
    ///     fn new(value: i32) -> PyResult<Self> {
    ///         Ok(Self { value })
    ///     }
    /// }
    ///
    /// #[pyclass(extends=BaseClass)]
    /// struct SubClass {}
    ///
    /// #[pymethods]
    /// impl SubClass {
    ///     #[new]
    ///     fn new(value: i32) -> PyResult<PyClassInitializer<Self>> {
    ///         let base_init = PyClassInitializer::from(BaseClass::new(value)?);
    ///         Ok(base_init.add_subclass(SubClass {}))
    ///     }
    /// }
    ///
    /// fn main() -> PyResult<()> {
    ///     Python::attach(|py| {
    ///         let m = PyModule::new(py, "example")?;
    ///         m.add_class::<SubClass>()?;
    ///         m.add_class::<BaseClass>()?;
    ///
    ///         let instance = m.getattr("SubClass")?.call1((92,))?;
    ///
    ///         // `SubClass` does not have a `value` attribute, but `BaseClass` does.
    ///         let n = instance.getattr("value")?.extract::<i32>()?;
    ///         assert_eq!(n, 92);
    ///
    ///         Ok(())
    ///     })
    /// }
    /// ```
    #[track_caller]
    #[inline]
    pub fn add_subclass<S>(self, subclass_value: S) -> PyClassInitializer<S>
    where
        S: PyClass<BaseType = T>,
        S::BaseType: PyClassBaseType<Initializer = Self>,
    {
        PyClassInitializer::new(subclass_value, self)
    }

    /// Creates a new class object and initializes it.
    pub(crate) fn create_class_object(self, py: Python<'_>) -> PyResult<Bound<'_, T>>
    where
        T: PyClass,
    {
        unsafe { self.create_class_object_of_type(py, T::type_object_raw(py)) }
    }

    /// Creates a new class object and initializes it given a typeobject `subtype`.
    ///
    /// # Safety
    /// `subtype` must be a valid pointer to the type object of T or a subclass.
    pub(crate) unsafe fn create_class_object_of_type(
        self,
        py: Python<'_>,
        target_type: *mut crate::ffi::PyTypeObject,
    ) -> PyResult<Bound<'_, T>>
    where
        T: PyClass,
    {
        /// Layout of a PyClassObject after base new has been called, but the contents have not yet been
        /// written.
        #[repr(C)]
        struct PartiallyInitializedClassObject<T: PyClass> {
            _ob_base: <T::BaseType as PyClassBaseType>::LayoutAsBase,
            contents: MaybeUninit<PyClassObjectContents<T>>,
        }

        let (init, super_init) = match self.0 {
            PyClassInitializerImpl::Existing(value) => return Ok(value.into_bound(py)),
            PyClassInitializerImpl::New { init, super_init } => (init, super_init),
        };

        let obj = unsafe { super_init.into_new_object(py, target_type)? };

        let part_init: *mut PartiallyInitializedClassObject<T> = obj.cast();
        unsafe {
            std::ptr::write(
                (*part_init).contents.as_mut_ptr(),
                PyClassObjectContents {
                    value: ManuallyDrop::new(UnsafeCell::new(init)),
                    borrow_checker: <T::PyClassMutability as PyClassMutability>::Storage::new(),
                    thread_checker: T::ThreadChecker::new(),
                    dict: T::Dict::INIT,
                    weakref: T::WeakRef::INIT,
                },
            );
        }

        // Safety: obj is a valid pointer to an object of type `target_type`, which` is a known
        // subclass of `T`
        Ok(unsafe { obj.assume_owned(py).cast_into_unchecked() })
    }
}

impl<T: PyClass> PyObjectInit<T> for PyClassInitializer<T> {
    unsafe fn into_new_object(
        self,
        py: Python<'_>,
        subtype: *mut PyTypeObject,
    ) -> PyResult<*mut ffi::PyObject> {
        unsafe {
            self.create_class_object_of_type(py, subtype)
                .map(Bound::into_ptr)
        }
    }

    #[inline]
    fn can_be_subclassed(&self) -> bool {
        !matches!(self.0, PyClassInitializerImpl::Existing(..))
    }
}

impl<T> From<T> for PyClassInitializer<T>
where
    T: PyClass,
    T::BaseType: PyClassBaseType<Initializer = PyNativeTypeInitializer<T::BaseType>>,
{
    #[inline]
    fn from(value: T) -> PyClassInitializer<T> {
        Self::new(value, PyNativeTypeInitializer(PhantomData))
    }
}

impl<S, B> From<(S, B)> for PyClassInitializer<S>
where
    S: PyClass<BaseType = B>,
    B: PyClass + PyClassBaseType<Initializer = PyClassInitializer<B>>,
    B::BaseType: PyClassBaseType<Initializer = PyNativeTypeInitializer<B::BaseType>>,
{
    #[track_caller]
    #[inline]
    fn from(sub_and_base: (S, B)) -> PyClassInitializer<S> {
        let (sub, base) = sub_and_base;
        PyClassInitializer::from(base).add_subclass(sub)
    }
}

impl<T: PyClass> From<Py<T>> for PyClassInitializer<T> {
    #[inline]
    fn from(value: Py<T>) -> PyClassInitializer<T> {
        PyClassInitializer(PyClassInitializerImpl::Existing(value))
    }
}

impl<'py, T: PyClass> From<Bound<'py, T>> for PyClassInitializer<T> {
    #[inline]
    fn from(value: Bound<'py, T>) -> PyClassInitializer<T> {
        PyClassInitializer::from(value.unbind())
    }
}

// Implementation used by proc macros to allow anything convertible to PyClassInitializer<T> to be
// the return value of pyclass #[new] method (optionally wrapped in `Result<U, E>`).
impl<T, U> IntoPyCallbackOutput<'_, PyClassInitializer<T>> for U
where
    T: PyClass,
    U: Into<PyClassInitializer<T>>,
{
    #[inline]
    fn convert(self, _py: Python<'_>) -> PyResult<PyClassInitializer<T>> {
        Ok(self.into())
    }
}

#[cfg(all(test, feature = "macros"))]
mod tests {
    //! See https://github.com/PyO3/pyo3/issues/4452.

    use crate::prelude::*;

    #[pyclass(crate = "crate", subclass)]
    struct BaseClass {}

    #[pyclass(crate = "crate", extends=BaseClass)]
    struct SubClass {
        _data: i32,
    }

    #[test]
    #[should_panic]
    fn add_subclass_to_py_is_unsound() {
        Python::attach(|py| {
            let base = Py::new(py, BaseClass {}).unwrap();
            let _subclass = PyClassInitializer::from(base).add_subclass(SubClass { _data: 42 });
        });
    }
}
