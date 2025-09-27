use pyo3::basic::CompareOp;
use pyo3::prelude::*;
use std::fmt;

#[pyclass(frozen)]
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

#[pyclass(frozen)]
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

#[pyclass(eq, frozen)]
#[derive(PartialEq, Eq)]
struct EqDerived(i64);

#[pymethods]
impl EqDerived {
    #[new]
    fn new(value: i64) -> Self {
        Self(value)
    }
}

#[pyclass(frozen)]
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

#[pyclass(frozen)]
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

#[pyclass(eq, ord, hash, str, frozen)]
#[derive(PartialEq, Eq, Ord, PartialOrd, Hash)]
struct OrderedDerived(i64);

impl fmt::Display for OrderedDerived {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

#[pymethods]
impl OrderedDerived {
    #[new]
    fn new(value: i64) -> Self {
        Self(value)
    }
}

#[pyclass(frozen)]
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
    use super::{
        Eq, EqDefaultNe, EqDerived, Ordered, OrderedDefaultNe, OrderedDerived, OrderedRichCmp,
    };
}
