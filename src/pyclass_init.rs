//! Contains initialization utilities for `#[pyclass]`.
use crate::callback::IntoPyCallbackOutput;
use crate::impl_::pyclass::{PyClassBaseType, PyClassDict, PyClassThreadChecker, PyClassWeakRef};
use crate::{ffi, Py, PyCell, PyClass, PyErr, PyResult, Python};
use crate::{
    ffi::PyTypeObject,
    pycell::{
        impl_::{PyClassBorrowChecker, PyClassMutability},
        PyCellContents,
    },
    type_object::{get_tp_alloc, PyTypeInfo},
};
use std::{
    cell::UnsafeCell,
    marker::PhantomData,
    mem::{ManuallyDrop, MaybeUninit},
};

/// Initializer for Python types.
///
/// This trait is intended to use internally for distinguishing `#[pyclass]` and
/// Python native types.
pub trait PyObjectInit<T>: Sized {
    /// # Safety
    /// - `subtype` must be a valid pointer to a type object of T or a subclass.
    unsafe fn into_new_object(
        self,
        py: Python<'_>,
        subtype: *mut PyTypeObject,
    ) -> PyResult<*mut ffi::PyObject>;
    private_decl! {}
}

/// Initializer for Python native types, like `PyDict`.
pub struct PyNativeTypeInitializer<T: PyTypeInfo>(PhantomData<T>);

impl<T: PyTypeInfo> PyObjectInit<T> for PyNativeTypeInitializer<T> {
    unsafe fn into_new_object(
        self,
        py: Python<'_>,
        subtype: *mut PyTypeObject,
    ) -> PyResult<*mut ffi::PyObject> {
        unsafe fn inner(
            py: Python<'_>,
            type_object: *mut PyTypeObject,
            subtype: *mut PyTypeObject,
        ) -> PyResult<*mut ffi::PyObject> {
            // HACK (due to FIXME below): PyBaseObject_Type's tp_new isn't happy with NULL arguments
            let is_base_object = type_object == std::ptr::addr_of_mut!(ffi::PyBaseObject_Type);
            if is_base_object {
                let alloc = get_tp_alloc(subtype).unwrap_or(ffi::PyType_GenericAlloc);
                let obj = alloc(subtype, 0);
                return if obj.is_null() {
                    Err(PyErr::fetch(py))
                } else {
                    Ok(obj)
                };
            }

            #[cfg(Py_LIMITED_API)]
            unreachable!("subclassing native types is not possible with the `abi3` feature");

            #[cfg(not(Py_LIMITED_API))]
            {
                match (*type_object).tp_new {
                    // FIXME: Call __new__ with actual arguments
                    Some(newfunc) => {
                        let obj = newfunc(subtype, std::ptr::null_mut(), std::ptr::null_mut());
                        if obj.is_null() {
                            Err(PyErr::fetch(py))
                        } else {
                            Ok(obj)
                        }
                    }
                    None => Err(crate::exceptions::PyTypeError::new_err(
                        "base type without tp_new",
                    )),
                }
            }
        }
        let type_object = T::type_object_raw(py);
        inner(py, type_object, subtype)
    }

    private_impl! {}
}

/// Initializer for our `#[pyclass]` system.
///
/// You can use this type to initialize complicatedly nested `#[pyclass]`.
///
/// # Examples
///
/// ```
/// # use pyo3::prelude::*;
/// # use pyo3::py_run_bound;
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
/// Python::with_gil(|py| {
///     let typeobj = py.get_type::<SubSubClass>();
///     let sub_sub_class = typeobj.call((), None).unwrap();
///     py_run_bound!(
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
    pub fn new(init: T, super_init: <T::BaseType as PyClassBaseType>::Initializer) -> Self {
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
    ///     Python::with_gil(|py| {
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
    pub fn add_subclass<S>(self, subclass_value: S) -> PyClassInitializer<S>
    where
        S: PyClass<BaseType = T>,
        S::BaseType: PyClassBaseType<Initializer = Self>,
    {
        PyClassInitializer::new(subclass_value, self)
    }

    /// Creates a new PyCell and initializes it.
    #[doc(hidden)]
    pub fn create_cell(self, py: Python<'_>) -> PyResult<*mut PyCell<T>>
    where
        T: PyClass,
    {
        unsafe { self.create_cell_from_subtype(py, T::type_object_raw(py)) }
    }

    /// Creates a new PyCell and initializes it given a typeobject `subtype`.
    /// Called by the Python `tp_new` implementation generated by a `#[new]` function in a `#[pymethods]` block.
    ///
    /// # Safety
    /// `subtype` must be a valid pointer to the type object of T or a subclass.
    #[doc(hidden)]
    pub unsafe fn create_cell_from_subtype(
        self,
        py: Python<'_>,
        subtype: *mut crate::ffi::PyTypeObject,
    ) -> PyResult<*mut PyCell<T>>
    where
        T: PyClass,
    {
        self.into_new_object(py, subtype).map(|obj| obj as _)
    }
}

impl<T: PyClass> PyObjectInit<T> for PyClassInitializer<T> {
    unsafe fn into_new_object(
        self,
        py: Python<'_>,
        subtype: *mut PyTypeObject,
    ) -> PyResult<*mut ffi::PyObject> {
        /// Layout of a PyCell after base new has been called, but the contents have not yet been
        /// written.
        #[repr(C)]
        struct PartiallyInitializedPyCell<T: PyClass> {
            _ob_base: <T::BaseType as PyClassBaseType>::LayoutAsBase,
            contents: MaybeUninit<PyCellContents<T>>,
        }

        let (init, super_init) = match self.0 {
            PyClassInitializerImpl::Existing(value) => return Ok(value.into_ptr()),
            PyClassInitializerImpl::New { init, super_init } => (init, super_init),
        };

        let obj = super_init.into_new_object(py, subtype)?;

        let cell: *mut PartiallyInitializedPyCell<T> = obj as _;
        std::ptr::write(
            (*cell).contents.as_mut_ptr(),
            PyCellContents {
                value: ManuallyDrop::new(UnsafeCell::new(init)),
                borrow_checker: <T::PyClassMutability as PyClassMutability>::Storage::new(),
                thread_checker: T::ThreadChecker::new(),
                dict: T::Dict::INIT,
                weakref: T::WeakRef::INIT,
            },
        );
        Ok(obj)
    }

    private_impl! {}
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
    B: PyClass,
    B::BaseType: PyClassBaseType<Initializer = PyNativeTypeInitializer<B::BaseType>>,
{
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

// Implementation used by proc macros to allow anything convertible to PyClassInitializer<T> to be
// the return value of pyclass #[new] method (optionally wrapped in `Result<U, E>`).
impl<T, U> IntoPyCallbackOutput<PyClassInitializer<T>> for U
where
    T: PyClass,
    U: Into<PyClassInitializer<T>>,
{
    #[inline]
    fn convert(self, _py: Python<'_>) -> PyResult<PyClassInitializer<T>> {
        Ok(self.into())
    }
}
