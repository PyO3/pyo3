#![cfg(feature = "macros")]

use std::sync::OnceLock;

use pyo3::create_exception;
use pyo3::exceptions::PyException;
use pyo3::prelude::*;
use pyo3::sync::OnceLockExt;

mod test_utils;

mod some_module {
    use pyo3::create_exception;
    use pyo3::exceptions::PyException;
    use pyo3::prelude::*;

    #[pyclass]
    pub struct SomePyClass;

    create_exception!(some_module, SomeException, PyException);
}

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
pub struct LocatedClass {}

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

#[pymodule(submodule)]
mod external_submodule {}

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

    // test for #4036
    #[pymodule_export]
    use super::some_module::SomePyClass;

    // test for #4036
    #[pymodule_export]
    use super::some_module::SomeException;

    #[pymodule_export]
    use super::external_submodule;

    #[pymodule_export]
    const FOO: u32 = 42;

    #[pymodule_export]
    #[cfg(Py_LIMITED_API)]
    const BAR: &str = "BAR";

    #[pymodule_export]
    #[allow(non_upper_case_globals)]
    const r#type: char = '!';

    #[allow(unused)]
    const NOT_EXPORTED: &str = "not exported";

    #[pymodule]
    mod inner {
        use super::*;

        #[pyfunction]
        fn triple(x: usize) -> usize {
            x * 3
        }

        #[pyclass(name = "Struct")]
        struct Struct;

        #[pymethods]
        impl Struct {
            #[new]
            fn new() -> Self {
                Self
            }
        }

        #[pyclass(module = "foo")]
        struct StructInCustomModule;

        #[pyclass(eq, eq_int, name = "Enum")]
        #[derive(PartialEq)]
        enum Enum {
            A,
            B,
        }

        #[pyclass(eq, eq_int, module = "foo")]
        #[derive(PartialEq)]
        enum EnumInCustomModule {
            A,
            B,
        }
    }

    #[pymodule]
    #[pyo3(module = "custom_root")]
    mod inner_custom_root {
        use super::*;

        #[pyclass]
        struct Struct;
    }

    #[pyo3::prelude::pymodule]
    mod full_path_inner {}

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

#[pymodule(name = "declarative_module_renamed")]
mod declarative_module2 {
    #[pymodule_export]
    use super::double;
}

fn declarative_module(py: Python<'_>) -> &Bound<'_, PyModule> {
    static MODULE: OnceLock<Py<PyModule>> = OnceLock::new();

    MODULE
        .get_or_init_py_attached(py, || pyo3::wrap_pymodule!(declarative_module)(py))
        .bind(py)
}

#[test]
fn test_declarative_module() {
    Python::attach(|py| {
        let m = declarative_module(py);
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
        py_assert!(py, m, "hasattr(m, 'external_submodule')");
        py_assert!(py, m, "m.FOO == 42");
        #[cfg(Py_LIMITED_API)]
        py_assert!(py, m, "m.BAR == 'BAR'");
        #[cfg(not(Py_LIMITED_API))]
        py_assert!(py, m, "not hasattr(m, 'BAR')");
        py_assert!(py, m, "m.type == '!'");
        py_assert!(py, m, "not hasattr(m, 'NOT_EXPORTED')");
    })
}

#[pymodule]
mod r#type {
    #[pymodule_export]
    use super::double;
}

#[test]
fn test_raw_ident_module() {
    Python::attach(|py| {
        let m = pyo3::wrap_pymodule!(r#type)(py).into_bound(py);
        py_assert!(py, m, "m.double(2) == 4");
    })
}

#[test]
fn test_module_names() {
    Python::attach(|py| {
        let m = declarative_module(py);
        py_assert!(
            py,
            m,
            "m.inner.Struct.__module__ == 'declarative_module.inner'"
        );
        py_assert!(py, m, "m.inner.StructInCustomModule.__module__ == 'foo'");
        py_assert!(
            py,
            m,
            "m.inner.Enum.__module__ == 'declarative_module.inner'"
        );
        py_assert!(py, m, "m.inner.EnumInCustomModule.__module__ == 'foo'");
        py_assert!(
            py,
            m,
            "m.inner_custom_root.Struct.__module__ == 'custom_root.inner_custom_root'"
        );
    })
}

#[test]
fn test_inner_module_full_path() {
    Python::attach(|py| {
        let m = declarative_module(py);
        py_assert!(py, m, "m.full_path_inner");
    })
}
