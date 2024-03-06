#![cfg(feature = "experimental-declarative-modules")]

use pyo3::create_exception;
use pyo3::exceptions::PyException;
use pyo3::prelude::*;

#[path = "../src/tests/common.rs"]
mod common;

#[pyclass]
struct ValueClass {
    value: usize,
}

#[pymethods]
impl ValueClass {
    #[new]
    fn new(value: usize) -> Self {
        Self { value }
    }
}

#[pyclass(module = "module")]
struct LocatedClass {}

#[pyfunction]
fn double(x: usize) -> usize {
    x * 2
}

create_exception!(
    declarative_module,
    MyError,
    PyException,
    "Some description."
);

/// A module written using declarative syntax.
#[pymodule]
mod declarative_module {
    #[pymodule_export]
    use super::declarative_submodule;
    #[pymodule_export]
    // This is not a real constraint but to test cfg attribute support
    #[cfg(not(Py_LIMITED_API))]
    use super::LocatedClass;
    use super::*;
    #[pymodule_export]
    use super::{declarative_module2, double, MyError, ValueClass as Value};

    #[pymodule]
    mod inner {
        use super::*;

        #[pyfunction]
        fn triple(x: usize) -> usize {
            x * 3
        }

        #[pyclass]
        struct Struct;

        #[pymethods]
        impl Struct {
            #[new]
            fn new() -> Self {
                Self
            }
        }

        #[pyclass]
        enum Enum {
            A,
            B,
        }
    }

    #[pymodule_init]
    fn init(m: &Bound<'_, PyModule>) -> PyResult<()> {
        m.add("double2", m.getattr("double")?)
    }
}

#[pyfunction]
fn double_value(v: &ValueClass) -> usize {
    v.value * 2
}

#[pymodule]
mod declarative_submodule {
    #[pymodule_export]
    use super::{double, double_value};
}

#[pymodule]
#[pyo3(name = "declarative_module_renamed")]
mod declarative_module2 {
    #[pymodule_export]
    use super::double;
}

#[test]
fn test_declarative_module() {
    Python::with_gil(|py| {
        let m = pyo3::wrap_pymodule!(declarative_module)(py).into_bound(py);
        py_assert!(
            py,
            m,
            "m.__doc__ == 'A module written using declarative syntax.'"
        );

        py_assert!(py, m, "m.double(2) == 4");
        py_assert!(py, m, "m.inner.triple(3) == 9");
        py_assert!(py, m, "m.declarative_submodule.double(4) == 8");
        py_assert!(
            py,
            m,
            "m.declarative_submodule.double_value(m.ValueClass(1)) == 2"
        );
        py_assert!(py, m, "str(m.MyError('foo')) == 'foo'");
        py_assert!(py, m, "m.declarative_module_renamed.double(2) == 4");
        #[cfg(Py_LIMITED_API)]
        py_assert!(py, m, "not hasattr(m, 'LocatedClass')");
        #[cfg(not(Py_LIMITED_API))]
        py_assert!(py, m, "hasattr(m, 'LocatedClass')");
        py_assert!(py, m, "isinstance(m.inner.Struct(), m.inner.Struct)");
        py_assert!(py, m, "isinstance(m.inner.Enum.A, m.inner.Enum)");
    })
}
