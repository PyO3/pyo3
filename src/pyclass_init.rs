//! Initialization utilities for `#[pyclass]`.
use crate::class::impl_::PyClassThreadChecker;
use crate::pyclass_slots::{PyClassDict, PyClassWeakRef};
use crate::{callback::IntoPyCallbackOutput, class::impl_::PyClassBaseType};
use crate::{ffi, PyCell, PyClass, PyErr, PyResult, Python};
use crate::{
    ffi::PyTypeObject,
    pycell::{BorrowFlag, PyCellContents},
    type_object::{get_tp_alloc, PyTypeInfo},
};
use std::{
    cell::{Cell, UnsafeCell},
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
        py: Python,
        subtype: *mut PyTypeObject,
    ) -> PyResult<*mut ffi::PyObject>;
    private_decl! {}
}

/// Initializer for Python native type, like `PyDict`.
pub struct PyNativeTypeInitializer<T: PyTypeInfo>(PhantomData<T>);

impl<T: PyTypeInfo> PyObjectInit<T> for PyNativeTypeInitializer<T> {
    unsafe fn into_new_object(
        self,
        py: Python,
        subtype: *mut PyTypeObject,
    ) -> PyResult<*mut ffi::PyObject> {
        let type_object = T::type_object_raw(py);

        // HACK (due to FIXME below): PyBaseObject_Type's tp_new isn't happy with NULL arguments
        if type_object == (&ffi::PyBaseObject_Type as *const _ as *mut _) {
            let alloc = get_tp_alloc(subtype).unwrap_or(ffi::PyType_GenericAlloc);
            let obj = alloc(subtype, 0);
            return if obj.is_null() {
                Err(PyErr::api_call_failed(py))
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
                        Err(PyErr::api_call_failed(py))
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

    private_impl! {}
}

/// Initializer for our `#[pyclass]` system.
///
/// You can use this type to initalize complicatedly nested `#[pyclass]`.
///
/// # Examples
///
/// ```
/// # use pyo3::prelude::*;
/// # use pyo3::py_run;
/// # use pyo3::types::IntoPyDict;
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
///             .add_subclass(SubSubClass { subsubname: "subsub" })
///     }
/// }
/// Python::with_gil(|py| {
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
pub struct PyClassInitializer<T: PyClass> {
    init: T,
    super_init: <T::BaseType as PyClassBaseType>::Initializer,
}

impl<T: PyClass> PyClassInitializer<T> {
    /// Construct new initializer from value `T` and base class' initializer.
    ///
    /// We recommend to mainly use `add_subclass`, instead of directly call `new`.
    pub fn new(init: T, super_init: <T::BaseType as PyClassBaseType>::Initializer) -> Self {
        Self { init, super_init }
    }

    /// Constructs a new initializer from base class' initializer.
    ///
    /// # Examples
    /// ```
    /// # use pyo3::prelude::*;
    /// #[pyclass]
    /// struct BaseClass {
    ///     value: u32,
    /// }
    ///
    /// impl BaseClass {
    ///     fn new(value: i32) -> PyResult<Self> {
    ///         Ok(Self {
    ///             value: std::convert::TryFrom::try_from(value)?,
    ///         })
    ///     }
    /// }
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
    /// ```
    pub fn add_subclass<S>(self, subclass_value: S) -> PyClassInitializer<S>
    where
        S: PyClass<BaseType = T>,
        S::BaseType: PyClassBaseType<Initializer = Self>,
    {
        PyClassInitializer::new(subclass_value, self)
    }

    /// Create a new PyCell and initialize it.
    #[doc(hidden)]
    pub fn create_cell(self, py: Python) -> PyResult<*mut PyCell<T>>
    where
        T: PyClass,
    {
        unsafe { self.create_cell_from_subtype(py, T::type_object_raw(py)) }
    }

    /// Create a new PyCell and initialize it given a typeobject `subtype`.
    /// Called by our `tp_new` generated by the `#[new]` attribute.
    ///
    /// # Safety
    /// `subtype` must be a valid pointer to the type object of T or a subclass.
    #[doc(hidden)]
    pub unsafe fn create_cell_from_subtype(
        self,
        py: Python,
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
        py: Python,
        subtype: *mut PyTypeObject,
    ) -> PyResult<*mut ffi::PyObject> {
        /// Layout of a PyCellBase after base new has been called, but borrow flag has not yet been
        /// initialized.
        #[repr(C)]
        struct PartiallyInitializedPyCellBase<T> {
            _ob_base: T,
            borrow_flag: MaybeUninit<Cell<BorrowFlag>>,
        }

        /// Layout of a PyCell after base new has been called, but contents have not yet been
        /// written.
        #[repr(C)]
        struct PartiallyInitializedPyCell<T: PyClass> {
            _ob_base: <T::BaseType as PyClassBaseType>::LayoutAsBase,
            contents: MaybeUninit<PyCellContents<T>>,
        }

        let Self { init, super_init } = self;
        let obj = super_init.into_new_object(py, subtype)?;

        // FIXME: Only need to initialize borrow flag once per whole hierarchy
        let base: *mut PartiallyInitializedPyCellBase<T::BaseNativeType> = obj as _;
        std::ptr::write(
            (*base).borrow_flag.as_mut_ptr(),
            Cell::new(BorrowFlag::UNUSED),
        );

        // FIXME: Initialize borrow flag if necessary??
        let cell: *mut PartiallyInitializedPyCell<T> = obj as _;
        std::ptr::write(
            (*cell).contents.as_mut_ptr(),
            PyCellContents {
                value: ManuallyDrop::new(UnsafeCell::new(init)),
                thread_checker: T::ThreadChecker::new(),
                dict: T::Dict::new(),
                weakref: T::WeakRef::new(),
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

// Implementation used by proc macros to allow anything convertible to PyClassInitializer<T> to be
// the return value of pyclass #[new] method (optionally wrapped in `Result<U, E>`).
impl<T, U> IntoPyCallbackOutput<PyClassInitializer<T>> for U
where
    T: PyClass,
    U: Into<PyClassInitializer<T>>,
{
    #[inline]
    fn convert(self, _py: Python) -> PyResult<PyClassInitializer<T>> {
        Ok(self.into())
    }
}
