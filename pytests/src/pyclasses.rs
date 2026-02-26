use std::{thread, time};

use pyo3::basic::CompareOp;
use pyo3::exceptions::{PyAttributeError, PyStopIteration, PyValueError};
use pyo3::prelude::*;
use pyo3::types::{PyComplex, PyType};
#[cfg(not(any(Py_LIMITED_API, GraalPy)))]
use pyo3::types::{PyDict, PyTuple};

#[pyclass(from_py_object)]
#[derive(Clone, Default)]
struct EmptyClass {}

#[pymethods]
impl EmptyClass {
    #[new]
    fn new() -> Self {
        EmptyClass {}
    }

    fn method(&self) {}

    fn __len__(&self) -> usize {
        0
    }
}

/// This is for demonstrating how to return a value from __next__
#[pyclass]
#[derive(Default)]
struct PyClassIter {
    count: usize,
}

#[pymethods]
impl PyClassIter {
    /// A constructor
    #[new]
    pub fn new() -> Self {
        Default::default()
    }

    fn __next__(&mut self) -> PyResult<usize> {
        if self.count < 5 {
            self.count += 1;
            Ok(self.count)
        } else {
            Err(PyStopIteration::new_err("Ended"))
        }
    }
}

#[pyclass]
#[derive(Default)]
struct PyClassThreadIter {
    count: usize,
}

#[pymethods]
impl PyClassThreadIter {
    #[new]
    pub fn new() -> Self {
        Default::default()
    }

    fn __next__(&mut self, py: Python<'_>) -> usize {
        let current_count = self.count;
        self.count += 1;
        if current_count == 0 {
            py.detach(|| thread::sleep(time::Duration::from_millis(100)));
        }
        self.count
    }
}

/// Demonstrates a base class which can operate on the relevant subclass in its constructor.
#[pyclass(subclass, skip_from_py_object)]
#[derive(Clone, Debug)]
struct AssertingBaseClass;

#[pymethods]
impl AssertingBaseClass {
    #[new]
    #[classmethod]
    fn new(cls: &Bound<'_, PyType>, expected_type: Bound<'_, PyType>) -> PyResult<Self> {
        if !cls.is(&expected_type) {
            return Err(PyValueError::new_err(format!(
                "{cls:?} != {expected_type:?}"
            )));
        }
        Ok(Self)
    }
}

#[pyclass]
struct ClassWithoutConstructor;

#[cfg(any(Py_3_10, not(Py_LIMITED_API)))]
#[pyclass(dict)]
struct ClassWithDict;

#[cfg(any(Py_3_10, not(Py_LIMITED_API)))]
#[pymethods]
impl ClassWithDict {
    #[new]
    fn new() -> Self {
        ClassWithDict
    }
}

#[cfg(not(any(Py_LIMITED_API, GraalPy)))] // Can't subclass native types on abi3 yet
#[pyclass(extends = PyDict)]
struct SubClassWithInit;

#[cfg(not(any(Py_LIMITED_API, GraalPy)))]
#[pymethods]
impl SubClassWithInit {
    #[new]
    #[pyo3(signature = (*args, **kwargs))]
    #[allow(unused_variables)]
    fn __new__(args: &Bound<'_, PyTuple>, kwargs: Option<&Bound<'_, PyDict>>) -> Self {
        Self
    }

    #[pyo3(signature = (*args, **kwargs))]
    fn __init__(
        self_: &Bound<'_, Self>,
        args: &Bound<'_, PyTuple>,
        kwargs: Option<&Bound<'_, PyDict>>,
    ) -> PyResult<()> {
        self_
            .py_super()?
            .call_method("__init__", args.to_owned(), kwargs)?;
        self_.as_super().set_item("__init__", true)?;
        Ok(())
    }
}

#[pyclass(skip_from_py_object)]
#[derive(Clone)]
struct ClassWithDecorators {
    attr: Option<usize>,
}

#[pymethods]
impl ClassWithDecorators {
    #[new]
    #[classmethod]
    fn new(_cls: Bound<'_, PyType>) -> Self {
        Self { attr: Some(0) }
    }

    /// A getter
    #[getter]
    fn get_attr(&self) -> PyResult<usize> {
        self.attr
            .ok_or_else(|| PyAttributeError::new_err("attr is not set"))
    }

    /// A setter
    #[setter]
    fn set_attr(&mut self, value: usize) {
        self.attr = Some(value);
    }

    /// A deleter
    #[deleter]
    fn delete_attr(&mut self) {
        self.attr = None;
    }

    /// A class method
    #[classmethod]
    fn cls_method(_cls: &Bound<'_, PyType>) -> usize {
        1
    }

    /// A static method
    #[staticmethod]
    fn static_method() -> usize {
        2
    }

    /// A class attribute
    #[classattr]
    fn cls_attribute() -> usize {
        3
    }
}

#[pyclass(get_all, set_all)]
struct PlainObject {
    /// Foo
    foo: String,
    /// Bar
    bar: usize,
}

#[derive(FromPyObject, IntoPyObject)]
enum AClass {
    NewType(EmptyClass),
    Tuple(EmptyClass, EmptyClass),
    Struct {
        f: EmptyClass,
        #[pyo3(item(42))]
        g: EmptyClass,
        #[pyo3(default)]
        h: EmptyClass,
    },
}

#[pyfunction]
fn map_a_class(cls: AClass) -> AClass {
    cls
}

#[pyclass]
struct Number(u64);

// TODO: Implementations are just for the example and often not correct
#[pymethods]
impl Number {
    #[new]
    fn new(value: u64) -> Self {
        Self(value)
    }

    fn __str__(&self) -> String {
        self.0.to_string()
    }

    fn __repr__(&self) -> String {
        format!("{:?}", self.0)
    }

    fn __hash__(&self) -> u64 {
        self.0
    }

    fn __richcmp__(&self, other: &Self, op: CompareOp) -> bool {
        op.matches(self.0.cmp(&other.0))
    }

    fn __add__(&self, other: &Self) -> Self {
        Self(self.0 + other.0)
    }

    fn __sub__(&self, other: &Self) -> Self {
        Self(self.0 - other.0)
    }

    fn __mul__(&self, other: &Self) -> Self {
        Self(self.0 * other.0)
    }

    fn __matmul__(&self, other: &Self) -> Self {
        Self(self.0 * other.0)
    }

    fn __truediv__(&self, other: &Self) -> Self {
        Self(self.0 / other.0)
    }

    fn __floordiv__(&self, other: &Self) -> Self {
        Self(self.0 / other.0)
    }

    fn __mod__(&self, other: &Self) -> Self {
        Self(self.0 % other.0)
    }

    fn __divmod__(&self, other: &Self) -> (Self, Self) {
        (Self(self.0 / other.0), Self(self.0 % other.0))
    }

    fn __pow__(&self, other: &Self, modulo: Option<&Self>) -> Self {
        Self(self.0.pow(other.0 as u32) % modulo.map_or(1, |m| m.0))
    }

    fn __rshift__(&self, other: &Self) -> Self {
        Self(self.0 >> other.0)
    }

    fn __lshift__(&self, other: &Self) -> Self {
        Self(self.0 << other.0)
    }

    fn __and__(&self, other: &Self) -> Self {
        Self(self.0 & other.0)
    }

    fn __or__(&self, other: &Self) -> Self {
        Self(self.0 | other.0)
    }

    fn __xor__(&self, other: &Self) -> Self {
        Self(self.0 ^ other.0)
    }

    fn __pos__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        slf
    }

    fn __neg__(&self) -> PyResult<Self> {
        if self.0 == 0 {
            Ok(Self(0))
        } else {
            Err(PyValueError::new_err("not zero"))
        }
    }

    fn __abs__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        slf
    }

    fn __invert__(&self) -> Self {
        Self(!self.0)
    }

    fn __int__(&self) -> u64 {
        self.0
    }

    fn __float__(&self) -> f64 {
        self.0 as f64
    }

    fn __complex__<'py>(&self, py: Python<'py>) -> Bound<'py, PyComplex> {
        PyComplex::from_doubles(py, self.0 as f64, 0.)
    }
}

#[pymodule]
pub mod pyclasses {
    #[cfg(any(Py_3_10, not(Py_LIMITED_API)))]
    #[pymodule_export]
    use super::ClassWithDict;
    #[cfg(not(any(Py_LIMITED_API, GraalPy)))]
    #[pymodule_export]
    use super::SubClassWithInit;
    #[pymodule_export]
    use super::{
        map_a_class, AssertingBaseClass, ClassWithDecorators, ClassWithoutConstructor, EmptyClass,
        Number, PlainObject, PyClassIter, PyClassThreadIter,
    };
}
