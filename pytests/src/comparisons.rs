use pyo3::basic::CompareOp;
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
struct OrderedRichCmp(i64);

#[pymethods]
impl OrderedRichCmp {
    #[new]
    fn new(value: i64) -> Self {
        Self(value)
    }

    fn __richcmp__(&self, other: &Self, op: CompareOp) -> bool {
        op.matches(self.0.cmp(&other.0))
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
pub mod comparisons {
    #[pymodule_export]
    use super::{Eq, EqDefaultNe, EqDerived, Ordered, OrderedDefaultNe, OrderedRichCmp};
}
