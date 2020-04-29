//! Initialization utilities for `#[pyclass]`.
use crate::pycell::PyCellLayout;
use crate::type_object::{PyBorrowFlagLayout, PyLayout, PySizedLayout, PyTypeInfo};
use crate::{PyClass, PyResult, Python};
use std::marker::PhantomData;

/// Initializer for Python types.
///
/// This trait is intended to use internally for distinguishing `#[pyclass]` and
/// Python native types.
pub trait PyObjectInit<'py, T: PyTypeInfo<'py>>: Sized {
    fn init_class<L: PyLayout<'py, T>>(self, layout: &mut L);
    private_decl! {}
}

/// Initializer for Python native type, like `PyDict`.
pub struct PyNativeTypeInitializer<'py, T: PyTypeInfo<'py>>(PhantomData<T>, PhantomData<Python<'py>>);

impl<'py, T: PyTypeInfo<'py>> PyObjectInit<'py, T> for PyNativeTypeInitializer<'py, T> {
    fn init_class<L: PyLayout<'py, T>>(self, _layout: &mut L) {}
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
pub struct PyClassInitializer<'py, T: PyClass<'py>> {
    init: T,
    super_init: <T::BaseType as PyTypeInfo<'py>>::Initializer,
}

impl<'py, T: PyClass<'py>> PyClassInitializer<'py, T> {
    /// Constract new initialzer from value `T` and base class' initializer.
    ///
    /// We recommend to mainly use `add_subclass`, instead of directly call `new`.
    pub fn new(init: T, super_init: <T::BaseType as PyTypeInfo<'py>>::Initializer) -> Self {
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
    pub fn add_subclass<S>(self, subclass_value: S) -> PyClassInitializer<'py, S>
    where
        S: PyClass<'py> + PyTypeInfo<'py, BaseType = T>,
        S::BaseLayout: PySizedLayout<'py, T>,
        S::BaseType: PyTypeInfo<'py, Initializer = Self>,
    {
        PyClassInitializer::new(subclass_value, self)
    }

    // Create a new PyCell + initialize it
    #[doc(hidden)]
    pub unsafe fn create_cell(self, py: Python<'py>) -> PyResult<*mut PyCellLayout<'py, T>>
    where
        T: PyClass<'py>,
        T::BaseLayout: PyBorrowFlagLayout<'py, T::BaseType>,
    {
        let cell = PyCellLayout::new(py)?;
        self.init_class(&mut *cell);
        Ok(cell)
    }
}

impl<'py, T: PyClass<'py>> PyObjectInit<'py, T> for PyClassInitializer<'py, T> {
    fn init_class<L: PyLayout<'py, T>>(self, layout: &mut L) {
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

impl<'py, T> From<T> for PyClassInitializer<'py, T>
where
    T: PyClass<'py>,
    T::BaseType: PyTypeInfo<'py, Initializer = PyNativeTypeInitializer<'py, T::BaseType>>,
{
    fn from(value: T) -> PyClassInitializer<'py, T> {
        Self::new(value, PyNativeTypeInitializer(PhantomData, PhantomData))
    }
}

impl<'py, S, B> From<(S, B)> for PyClassInitializer<'py, S>
where
    S: PyClass<'py> + PyTypeInfo<'py, BaseType = B>,
    S::BaseLayout: PySizedLayout<'py, B>,
    B: PyClass<'py> + PyTypeInfo<'py, Initializer = PyClassInitializer<'py, B>>,
    B::BaseType: PyTypeInfo<'py, Initializer = PyNativeTypeInitializer<'py, B::BaseType>>,
{
    fn from(sub_and_base: (S, B)) -> PyClassInitializer<'py, S> {
        let (sub, base) = sub_and_base;
        PyClassInitializer::from(base).add_subclass(sub)
    }
}
