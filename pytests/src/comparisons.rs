use pyo3::prelude::*;

#[pyclass]
struct Eq(i64);

#[pymethods]
impl Eq {
    #[new]
    fn new(value: i64) -> Self {
        Self(value)
    }

    fn __eq__(&self, other: &Self) -> bool {
        self.0 == other.0
    }

    fn __ne__(&self, other: &Self) -> bool {
        self.0 != other.0
    }
}

#[pyclass]
struct EqDefaultNe(i64);

#[pymethods]
impl EqDefaultNe {
    #[new]
    fn new(value: i64) -> Self {
        Self(value)
    }

    fn __eq__(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

#[pyclass(eq)]
#[derive(PartialEq, Eq)]
struct EqDerived(i64);

#[pymethods]
impl EqDerived {
    #[new]
    fn new(value: i64) -> Self {
        Self(value)
    }
}

#[pyclass]
struct Ordered(i64);

#[pymethods]
impl Ordered {
    #[new]
    fn new(value: i64) -> Self {
        Self(value)
    }

    fn __lt__(&self, other: &Self) -> bool {
        self.0 < other.0
    }

    fn __le__(&self, other: &Self) -> bool {
        self.0 <= other.0
    }

    fn __eq__(&self, other: &Self) -> bool {
        self.0 == other.0
    }

    fn __ne__(&self, other: &Self) -> bool {
        self.0 != other.0
    }

    fn __gt__(&self, other: &Self) -> bool {
        self.0 > other.0
    }

    fn __ge__(&self, other: &Self) -> bool {
        self.0 >= other.0
    }
}

#[pyclass]
struct OrderedDefaultNe(i64);

#[pymethods]
impl OrderedDefaultNe {
    #[new]
    fn new(value: i64) -> Self {
        Self(value)
    }

    fn __lt__(&self, other: &Self) -> bool {
        self.0 < other.0
    }

    fn __le__(&self, other: &Self) -> bool {
        self.0 <= other.0
    }

    fn __eq__(&self, other: &Self) -> bool {
        self.0 == other.0
    }

    fn __gt__(&self, other: &Self) -> bool {
        self.0 > other.0
    }

    fn __ge__(&self, other: &Self) -> bool {
        self.0 >= other.0
    }
}

#[pymodule(gil_used = false)]
pub fn comparisons(m: &Bound<'_, PyModule>) -> PyResult<()> {
    PyModule::add_class::<Eq>(m)?;
    PyModule::add_class::<EqDefaultNe>(m)?;
    PyModule::add_class::<EqDerived>(m)?;
    PyModule::add_class::<Ordered>(m)?;
    PyModule::add_class::<OrderedDefaultNe>(m)?;
    Ok(())
}
