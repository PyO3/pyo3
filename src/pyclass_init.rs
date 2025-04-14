//! Contains initialization utilities for `#[pyclass]`.
//!
//! # Background
//!
//! Initialization of a regular empty python class `class MyClass: pass`
//! - `MyClass(*args, **kwargs) == MyClass.__call__(*args, **kwargs) == type.__call__(*args, **kwargs)`
//!   - `MyClass.tp_call` is NULL but `obj.__call__` uses `Py_TYPE(obj)->tp_call` ([ffi::_PyObject_MakeTpCall])
//!     - so `__call__` is inherited from the metaclass which is `type` in this case.
//!   - `type.__call__` calls `obj = MyClass.__new__(MyClass, *args, **kwargs) == object.__new__(MyClass, *args, **kwargs)`
//!     - `MyClass.tp_new` is inherited from the base class which is `object` in this case.
//!     - Allocates a new object and does some basic initialization
//!   - `type.__call__` calls `MyClass.__init__(obj, *args, **kwargs) == object.__init__(obj, *args, **kwargs)`
//!     - `MyClass.tp_init` is inherited from the base class which is `object` in this case.
//!     - Does some checks but is essentially a no-op
//!
//! So in general for `class MyClass(BaseClass, metaclass=MetaClass): pass`
//! - `MyClass(*args, **kwargs)`
//!   - `Metaclass.__call__(*args, **kwargs)`
//!     - `BaseClass.__new__(*args, **kwargs)`
//!     - `BaseClass.__init__(*args, **kwargs)`
//!
//! - If `MyClass` defines `__new__` then it must delegate to `super(MyClass, cls).__new__(cls, *args, **kwargs)` to
//!   allocate the object.
//!   - is is the responsibility of `MyClass` to call `super().__new__()` with the correct arguments.
//!     `object.__new__()` does not accept any arguments for example.
//! - If `MyClass` defines `__init__` then it should call `super().__init__()` to recursively initialize
//!   the base classes. Again, passing arguments is the responsibility of MyClass.
//!
//! Initialization of a pyo3 `#[pyclass] struct MyClass;`
//! - `MyClass(*args, **kwargs) == MyClass.__call__(*args, **kwargs) == type.__call__(*args, **kwargs)`
//!   - Calls `obj = MyClass.__new__(MyClass, *args, **kwargs)`
//!     - Calls user defined `#[new]` function, returning a [`IntoPyCallbackOutput<PyClassInitializer>`] which has
//!       instances of each user defined struct in the inheritance hierarchy.
//!     - Calls `PyClassInitializer::create_class_object_of_type`
//!       - Recursively calls back to the base native type.
//!       - At the base native type, [PyObjectInit::into_new_object] calls `__new__` for the base native type
//!         (passing the [ffi::PyTypeObject] of the most derived type)
//!         - Allocates a new python object with enough space to fit the user structs and the native base type data.
//!         - Initializes the native base type part of the new object.
//!       - Moves the data for the user structs into the appropriate locations in the new python object.
//!   - Calls `MyClass.__init__(obj, *args, **kwargs)`
//!     - Inherited from native base class
//!
//! ## Notes:
//! - pyo3 classes annotated with `#[pyclass(dict)]` have a `__dict__` attribute. When using the `tp_dictoffset`
//!   mechanism instead of `Py_TPFLAGS_MANAGED_DICT` to enable this, the dict is stored in the `PyClassObjectContents`
//!   of the most derived type and is set to NULL at construction and initialized to a new dictionary by
//!   [ffi::PyObject_GenericGetDict] when first accessed.
//! - The python documentation also mentions 'static' classes which define their [ffi::PyTypeObject] in static/global
//!   memory. Builtins like `dict` (`PyDict_Type`) are defined this way but classes defined in python and pyo3 are
//!   'heap' types where the [ffi::PyTypeObject] objects are allocated at runtime.
//!
use crate::ffi_ptr_ext::FfiPtrExt;
use crate::impl_::callback::IntoPyCallbackOutput;
use crate::impl_::pyclass::{PyClassBaseType, PyClassDict, PyClassThreadChecker, PyClassWeakRef};
use crate::impl_::pyclass_init::{PyNativeTypeInitializer, PyObjectInit};
use crate::types::{PyAnyMethods, PyDict, PyTuple};
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
    #[track_caller]
    #[inline]
    pub fn add_subclass<S>(self, subclass_value: S) -> PyClassInitializer<S>
    where
        S: PyClass<BaseType = T>,
        S::BaseType: PyClassBaseType<Initializer = Self>,
    {
        PyClassInitializer::new(subclass_value, self)
    }

    /// Creates a new PyCell and initializes it.
    pub(crate) fn create_class_object(self, py: Python<'_>) -> PyResult<Bound<'_, T>>
    where
        T: PyClass,
    {
        unsafe {
            self.create_class_object_of_type(py, T::type_object_raw(py), &PyTuple::empty(py), None)
        }
    }

    /// Creates a new class object and initializes it given a typeobject `subtype`.
    ///
    /// # Safety
    /// `subtype` must be a valid pointer to the type object of T or a subclass.
    pub(crate) unsafe fn create_class_object_of_type<'py>(
        self,
        py: Python<'py>,
        target_type: *mut crate::ffi::PyTypeObject,
        args: &Bound<'py, PyTuple>,
        kwargs: Option<&Bound<'py, PyDict>>,
    ) -> PyResult<Bound<'py, T>>
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

        let obj = unsafe { super_init.into_new_object(py, target_type, kwargs)? };

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
        Ok(unsafe { obj.assume_owned(py).downcast_into_unchecked() })
    }
}

impl<T: PyClass> PyObjectInit<T> for PyClassInitializer<T> {
    unsafe fn into_new_object(
        self,
        py: Python<'_>,
        subtype: *mut PyTypeObject,
        args: &Bound<'_, PyTuple>,
        kwargs: Option<&Bound<'_, PyDict>>,
    ) -> PyResult<*mut ffi::PyObject> {
        unsafe {
            self.create_class_object_of_type(py, subtype, args, kwargs)
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
    use crate::{
        ffi,
        prelude::*,
        types::{PyDict, PyType},
        PyTypeInfo,
    };

    #[pyclass(crate = "crate", subclass)]
    struct BaseClass {}

    #[pyclass(crate = "crate", extends=BaseClass)]
    struct SubClass {
        _data: i32,
    }

    /// See https://github.com/PyO3/pyo3/issues/4452.
    #[test]
    #[should_panic(expected = "you cannot add a subclass to an existing value")]
    fn add_subclass_to_py_is_unsound() {
        Python::with_gil(|py| {
            let base = Py::new(py, BaseClass {}).unwrap();
            let _subclass = PyClassInitializer::from(base).add_subclass(SubClass { _data: 42 });
        });
    }

    /// Verify the correctness of the documentation describing class initialization.
    #[test]
    fn empty_class_inherits_expected_slots() {
        Python::with_gil(|py| {
            let namespace = PyDict::new(py);
            py.run(
                ffi::c_str!("class EmptyClass: pass"),
                Some(&namespace),
                Some(&namespace),
            )
            .unwrap();
            let empty_class = namespace
                .get_item("EmptyClass")
                .unwrap()
                .unwrap()
                .downcast::<PyType>()
                .unwrap()
                .as_type_ptr();

            let object_type = PyAny::type_object(py).as_type_ptr();
            unsafe {
                assert_eq!(
                    ffi::PyType_GetSlot(empty_class, ffi::Py_tp_new),
                    ffi::PyType_GetSlot(object_type, ffi::Py_tp_new)
                );
                assert_eq!(
                    ffi::PyType_GetSlot(empty_class, ffi::Py_tp_init),
                    ffi::PyType_GetSlot(object_type, ffi::Py_tp_init)
                );
                assert!(ffi::PyType_GetSlot(empty_class, ffi::Py_tp_call).is_null());
            }

            let base_class = BaseClass::type_object_raw(py);
            unsafe {
                // tp_new is always set for pyclasses, not inherited
                assert_ne!(
                    ffi::PyType_GetSlot(base_class, ffi::Py_tp_new),
                    ffi::PyType_GetSlot(object_type, ffi::Py_tp_new)
                );
                assert_eq!(
                    ffi::PyType_GetSlot(base_class, ffi::Py_tp_init),
                    ffi::PyType_GetSlot(object_type, ffi::Py_tp_init)
                );
                assert!(ffi::PyType_GetSlot(base_class, ffi::Py_tp_call).is_null());
            }
        });
    }

    /// Verify the correctness of the documentation describing class initialization.
    #[test]
    #[cfg(all(Py_3_10, not(Py_LIMITED_API)))]
    fn managed_dict_initialized_as_expected() {
        use crate::{impl_::pyclass::PyClassImpl, types::PyFloat};

        unsafe fn get_dict<T: PyClassImpl>(obj: *mut ffi::PyObject) -> *mut *mut ffi::PyObject {
            let dict_offset = T::dict_offset().unwrap();
            (obj as *mut u8).add(usize::try_from(dict_offset).unwrap()) as *mut *mut ffi::PyObject
        }

        #[pyclass(crate = "crate", dict)]
        struct ClassWithDict {}

        Python::with_gil(|py| {
            let obj = Py::new(py, ClassWithDict {}).unwrap();
            unsafe {
                let obj_dict = get_dict::<ClassWithDict>(obj.as_ptr());
                assert!((*obj_dict).is_null());
                crate::py_run!(py, obj, "obj.__dict__");
                assert!(!(*obj_dict).is_null());
            }
        });

        #[pyclass(crate = "crate", dict, extends=PyFloat)]
        struct ExtendedClassWithDict {}

        Python::with_gil(|py| {
            let obj = Py::new(py, ExtendedClassWithDict {}).unwrap();
            unsafe {
                let obj_dict = get_dict::<ExtendedClassWithDict>(obj.as_ptr());
                assert!((*obj_dict).is_null());
                crate::py_run!(py, obj, "obj.__dict__");
                assert!(!(*obj_dict).is_null());
            }
        });
    }
}
