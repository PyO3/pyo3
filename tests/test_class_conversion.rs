#![cfg(feature = "macros")]

use pyo3::prelude::*;

#[macro_use]
mod test_utils;

#[pyclass]
#[derive(Clone, Debug, PartialEq)]
struct Cloneable {
    x: i32,
}

#[test]
fn test_cloneable_pyclass() {
    let c = Cloneable { x: 10 };

    Python::attach(|py| {
        let py_c = Py::new(py, c.clone()).unwrap();

        let c2: Cloneable = py_c.extract(py).unwrap();
        assert_eq!(c, c2);
        {
            let rc: PyRef<'_, Cloneable> = py_c.extract(py).unwrap();
            assert_eq!(&c, &*rc);
            // Drops PyRef before taking PyRefMut
        }
        let mrc: PyRefMut<'_, Cloneable> = py_c.extract(py).unwrap();
        assert_eq!(&c, &*mrc);
    });
}

#[pyclass(subclass)]
#[derive(Default)]
struct BaseClass {
    value: i32,
}

#[pymethods]
impl BaseClass {
    fn foo(&self) -> &'static str {
        "BaseClass"
    }
}

#[pyclass(extends=BaseClass)]
struct SubClass {}

#[pymethods]
impl SubClass {
    fn foo(&self) -> &'static str {
        "SubClass"
    }
}

#[pyclass]
struct PolymorphicContainer {
    #[pyo3(get, set)]
    inner: Py<BaseClass>,
}

#[test]
fn test_polymorphic_container_stores_base_class() {
    Python::attach(|py| {
        let p = Py::new(
            py,
            PolymorphicContainer {
                inner: Py::new(py, BaseClass::default()).unwrap(),
            },
        )
        .unwrap();

        py_assert!(py, p, "p.inner.foo() == 'BaseClass'");
    });
}

#[test]
fn test_polymorphic_container_stores_sub_class() {
    Python::attach(|py| {
        let p = Py::new(
            py,
            PolymorphicContainer {
                inner: Py::new(py, BaseClass::default()).unwrap(),
            },
        )
        .unwrap();

        p.bind(py)
            .setattr(
                "inner",
                Py::new(
                    py,
                    PyClassInitializer::from(BaseClass::default()).add_subclass(SubClass {}),
                )
                .unwrap(),
            )
            .unwrap();

        py_assert!(py, p, "p.inner.foo() == 'SubClass'");
    });
}

#[test]
fn test_polymorphic_container_does_not_accept_other_types() {
    Python::attach(|py| {
        let p = Py::new(
            py,
            PolymorphicContainer {
                inner: Py::new(py, BaseClass::default()).unwrap(),
            },
        )
        .unwrap();

        let setattr = |value: Bound<'_, PyAny>| p.bind(py).setattr("inner", value);

        assert!(setattr(1i32.into_pyobject(py).unwrap().into_any()).is_err());
        assert!(setattr(py.None().into_bound(py)).is_err());
        assert!(setattr((1i32, 2i32).into_pyobject(py).unwrap().into_any()).is_err());
    });
}

#[test]
fn test_pyref_as_base() {
    Python::attach(|py| {
        let cell = Bound::new(py, (SubClass {}, BaseClass { value: 120 })).unwrap();

        // First try PyRefMut
        let sub: PyRefMut<'_, SubClass> = cell.borrow_mut();
        let mut base: PyRefMut<'_, BaseClass> = sub.into_super();
        assert_eq!(120, base.value);
        base.value = 999;
        assert_eq!(999, base.value);
        drop(base);

        // Repeat for PyRef
        let sub: PyRef<'_, SubClass> = cell.borrow();
        let base: PyRef<'_, BaseClass> = sub.into_super();
        assert_eq!(999, base.value);
    });
}

#[test]
fn test_pycell_deref() {
    Python::attach(|py| {
        let obj = Bound::new(py, (SubClass {}, BaseClass { value: 120 })).unwrap();

        // Should be able to deref as PyAny
        assert_eq!(
            obj.call_method0("foo")
                .and_then(|e| e.extract::<String>())
                .unwrap(),
            "SubClass"
        );
    });
}
