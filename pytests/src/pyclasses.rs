use std::{thread, time};

use pyo3::exceptions::{PyAttributeError, PyStopIteration, PyValueError};
use pyo3::prelude::*;
use pyo3::types::PyType;
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
        PlainObject, PyClassIter, PyClassThreadIter,
    };
}
