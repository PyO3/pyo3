//! Initialization utilities for `#[pyclass]`.
use crate::type_object::{PyBorrowFlagLayout, PyLayout, PySizedLayout, PyTypeInfo};
use crate::{PyCell, PyClass, PyResult, Python};
use std::marker::PhantomData;

/// Initializer for Python types.
///
/// This trait is intended to use internally for distinguishing `#[pyclass]` and
/// Python native types.
pub trait PyObjectInit<T: PyTypeInfo>: Sized {
    fn init_class<L: PyLayout<T>>(self, layout: &mut L);
    private_decl! {}
}

/// Initializer for Python native type, like `PyDict`.
pub struct PyNativeTypeInitializer<T: PyTypeInfo>(PhantomData<T>);

impl<T: PyTypeInfo> PyObjectInit<T> for PyNativeTypeInitializer<T> {
    fn init_class<L: PyLayout<T>>(self, _layout: &mut L) {}
    private_impl! {}
}

/// Initializer for our `#[pyclass]` system.
///
/// You can use this type to initalize complicatedly nested `#[pyclass]`.
///
/// # Example
///
/// ```
/// # use pyo3::prelude::*;
/// # use pyo3::py_run;
/// #[pyclass]
/// struct BaseClass {
///     #[pyo3(get)]
///     basename: &'static str,
/// }
/// #[pyclass(extends=BaseClass)]
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
/// let gil = Python::acquire_gil();
/// let py = gil.python();
/// let typeobj = py.get_type::<SubSubClass>();
/// let inst = typeobj.call((), None).unwrap();
/// py_run!(py, inst, r#"
///         assert inst.basename == 'base'
///         assert inst.subname == 'sub'
///         assert inst.subsubname == 'subsub'"#);
/// ```
pub struct PyClassInitializer<T: PyClass> {
    init: T,
    super_init: <T::BaseType as PyTypeInfo>::Initializer,
}

impl<T: PyClass> PyClassInitializer<T> {
    /// Constract new initialzer from value `T` and base class' initializer.
    ///
    /// We recommend to mainly use `add_subclass`, instead of directly call `new`.
    pub fn new(init: T, super_init: <T::BaseType as PyTypeInfo>::Initializer) -> Self {
        Self { init, super_init }
    }

    /// Constructs a new initializer from base class' initializer.
    ///
    /// # Example
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
        S: PyClass + PyTypeInfo<BaseType = T>,
        S::BaseLayout: PySizedLayout<T>,
        S::BaseType: PyTypeInfo<Initializer = Self>,
    {
        PyClassInitializer::new(subclass_value, self)
    }

    // Create a new PyCell + initialize it
    #[doc(hidden)]
    pub unsafe fn create_cell(self, py: Python) -> PyResult<*mut PyCell<T>>
    where
        T: PyClass,
        T::BaseLayout: PyBorrowFlagLayout<T::BaseType>,
    {
        let cell = PyCell::internal_new(py)?;
        self.init_class(&mut *cell);
        Ok(cell)
    }
}

impl<T: PyClass> PyObjectInit<T> for PyClassInitializer<T> {
    fn init_class<L: PyLayout<T>>(self, layout: &mut L) {
        let Self { init, super_init } = self;
        unsafe {
            layout.py_init(init);
        }
        if let Some(super_obj) = layout.get_super() {
            super_init.init_class(super_obj);
        }
    }
    private_impl! {}
}

impl<T> From<T> for PyClassInitializer<T>
where
    T: PyClass,
    T::BaseType: PyTypeInfo<Initializer = PyNativeTypeInitializer<T::BaseType>>,
{
    fn from(value: T) -> PyClassInitializer<T> {
        Self::new(value, PyNativeTypeInitializer(PhantomData))
    }
}

impl<S, B> From<(S, B)> for PyClassInitializer<S>
where
    S: PyClass + PyTypeInfo<BaseType = B>,
    S::BaseLayout: PySizedLayout<B>,
    B: PyClass + PyTypeInfo<Initializer = PyClassInitializer<B>>,
    B::BaseType: PyTypeInfo<Initializer = PyNativeTypeInitializer<B::BaseType>>,
{
    fn from(sub_and_base: (S, B)) -> PyClassInitializer<S> {
        let (sub, base) = sub_and_base;
        PyClassInitializer::from(base).add_subclass(sub)
    }
}
