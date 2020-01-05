use crate::pyclass::{PyClass, PyClassShell};
use crate::type_object::{PyObjectLayout, PyObjectSizedLayout, PyTypeInfo};
use crate::{PyResult, Python};
use std::marker::PhantomData;

pub trait PyObjectInit<T: PyTypeInfo>: Sized {
    fn init_class(self, shell: &mut T::ConcreteLayout);
}

pub struct PyNativeTypeInitializer<T: PyTypeInfo>(PhantomData<T>);

impl<T: PyTypeInfo> PyObjectInit<T> for PyNativeTypeInitializer<T> {
    fn init_class(self, _shell: &mut T::ConcreteLayout) {}
}

/// An initializer for `PyClassShell<T>`.
///
/// You can use this type to initalize complicatedly nested `#[pyclass]`.
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
///         let base_init = PyClassInitializer::from_value(BaseClass{basename: "base"});
///         base_init.add_subclass(SubClass{subname: "sub"})
///                  .add_subclass(SubSubClass{subsubname: "subsub"})
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
    super_init: Box<<T::BaseType as PyTypeInfo>::Initializer>,
}

impl<T: PyClass> PyClassInitializer<T> {
    pub fn new(init: T, super_init: <T::BaseType as PyTypeInfo>::Initializer) -> Self {
        Self {
            init,
            super_init: Box::new(super_init),
        }
    }

    pub fn add_subclass<S>(self, subclass_value: S) -> PyClassInitializer<S>
    where
        S: PyClass + PyTypeInfo<BaseType = T>,
        S::BaseType: PyTypeInfo<Initializer = Self>,
    {
        PyClassInitializer::new(subclass_value, self)
    }

    #[doc(hidden)]
    pub unsafe fn create_shell(self, py: Python) -> PyResult<*mut PyClassShell<T>>
    where
        T: PyClass,
        <T::BaseType as PyTypeInfo>::ConcreteLayout: PyObjectSizedLayout<T::BaseType>,
    {
        let shell = PyClassShell::new(py)?;
        self.init_class(&mut *shell);
        Ok(shell)
    }
}

impl<T> PyClassInitializer<T>
where
    T: PyClass,
    T::BaseType: PyTypeInfo<Initializer = PyNativeTypeInitializer<T::BaseType>>,
{
    pub fn from_value(value: T) -> Self {
        Self::new(value, PyNativeTypeInitializer(PhantomData))
    }
}

impl<T: PyClass> PyObjectInit<T> for PyClassInitializer<T> {
    fn init_class(self, obj: &mut T::ConcreteLayout) {
        let Self { init, super_init } = self;
        unsafe {
            obj.py_init(init);
        }
        if let Some(super_obj) = obj.get_super_or() {
            super_init.init_class(super_obj);
        }
    }
}

/// Represets that we can convert the type to `PyClassInitializer`.
///
/// It is mainly used in our proc-macro code.
pub trait IntoInitializer<T: PyClass> {
    fn into_initializer(self) -> PyResult<PyClassInitializer<T>>;
}

impl<T> IntoInitializer<T> for T
where
    T: PyClass,
    T::BaseType: PyTypeInfo<Initializer = PyNativeTypeInitializer<T::BaseType>>,
{
    fn into_initializer(self) -> PyResult<PyClassInitializer<T>> {
        Ok(PyClassInitializer::from_value(self))
    }
}

impl<T> IntoInitializer<T> for PyResult<T>
where
    T: PyClass,
    T::BaseType: PyTypeInfo<Initializer = PyNativeTypeInitializer<T::BaseType>>,
{
    fn into_initializer(self) -> PyResult<PyClassInitializer<T>> {
        self.map(PyClassInitializer::from_value)
    }
}

impl<S, B> IntoInitializer<S> for (S, B)
where
    S: PyClass + PyTypeInfo<BaseType = B>,
    B: PyClass + PyTypeInfo<Initializer = PyClassInitializer<B>>,
    B::BaseType: PyTypeInfo<Initializer = PyNativeTypeInitializer<B::BaseType>>,
{
    fn into_initializer(self) -> PyResult<PyClassInitializer<S>> {
        let (sub, base) = self;
        Ok(PyClassInitializer::from_value(base).add_subclass(sub))
    }
}

impl<S, B> IntoInitializer<S> for PyResult<(S, B)>
where
    S: PyClass + PyTypeInfo<BaseType = B>,
    B: PyClass + PyTypeInfo<Initializer = PyClassInitializer<B>>,
    B::BaseType: PyTypeInfo<Initializer = PyNativeTypeInitializer<B::BaseType>>,
{
    fn into_initializer(self) -> PyResult<PyClassInitializer<S>> {
        self.map(|(sub, base)| PyClassInitializer::from_value(base).add_subclass(sub))
    }
}

impl<T: PyClass> IntoInitializer<T> for PyClassInitializer<T> {
    fn into_initializer(self) -> PyResult<PyClassInitializer<T>> {
        Ok(self)
    }
}

impl<T: PyClass> IntoInitializer<T> for PyResult<PyClassInitializer<T>> {
    fn into_initializer(self) -> PyResult<PyClassInitializer<T>> {
        self
    }
}
