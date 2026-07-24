#![cfg(feature = "macros")]

use pyo3::prelude::*;
use pyo3::py_run;
use pyo3::types::IntoPyDict;

mod test_utils;

/// Macro to generate refcount leak tests for types.
/// Ensures that creating and destroying instances doesn't leak references to the type.
/// Regression test for issues #1363 and #6223.
macro_rules! assert_type_refcount_stable {
    // Simple case: type with parameterless constructor
    ($type_name:ty) => {
        assert_type_refcount_stable!($type_name, stringify!($type_name), "Type()");
    };
    // With custom constructor
    ($type_name:ty, $test_name:expr, $ctor:expr) => {{
        Python::attach(|py| {
            #[expect(non_snake_case)]
            let Type = py.get_type::<$type_name>();
            let ctor_code = $ctor;
            py_run!(
                py,
                Type,
                &format!(
                    r#"
                        import gc
                        import sys

                        gc.collect()
                        count = sys.getrefcount(Type)

                        for i in range(1000):
                            obj = {}
                            del obj

                        gc.collect()
                        after = sys.getrefcount(Type)
                        assert after == count, f"Type ref count leaked: {{after}} vs {{count}}"
                        "#,
                    ctor_code
                )
            );
        });
    }};
}

#[pyclass(subclass)]
struct BaseClass {
    #[pyo3(get)]
    val1: usize,
}

#[pyclass(subclass)]
struct SubclassAble {}

#[test]
fn subclass() {
    Python::attach(|py| {
        let d = [("SubclassAble", py.get_type::<SubclassAble>())]
            .into_py_dict(py)
            .unwrap();

        py.run(
            c"class A(SubclassAble): pass\nassert issubclass(A, SubclassAble)",
            None,
            Some(&d),
        )
        .map_err(|e| e.display(py))
        .unwrap();
    });
}

#[pymethods]
impl BaseClass {
    #[new]
    fn new() -> Self {
        BaseClass { val1: 10 }
    }
    fn base_method(&self, x: usize) -> usize {
        x * self.val1
    }
    fn base_set(&mut self, fn_: &Bound<'_, PyAny>) -> PyResult<()> {
        let value: usize = fn_.call0()?.extract()?;
        self.val1 = value;
        Ok(())
    }
}

#[pyclass(extends=BaseClass)]
struct SubClass {
    #[pyo3(get)]
    val2: usize,
}

#[pymethods]
impl SubClass {
    #[new]
    fn new() -> PyClassInitializer<Self> {
        PyClassInitializer::from(BaseClass { val1: 10 }).add_subclass(SubClass { val2: 5 })
    }
    fn sub_method(&self, x: usize) -> usize {
        x * self.val2
    }
    fn sub_set_and_ret(&mut self, x: usize) -> usize {
        self.val2 = x;
        x
    }
}

#[test]
fn inheritance_with_new_methods() {
    Python::attach(|py| {
        let typeobj = py.get_type::<SubClass>();
        let inst = typeobj.call((), None).unwrap();
        py_run!(py, inst, "assert inst.val1 == 10; assert inst.val2 == 5");
    });
}

#[test]
fn call_base_and_sub_methods() {
    Python::attach(|py| {
        let obj = Py::new(py, SubClass::new()).unwrap();
        py_run!(
            py,
            obj,
            r#"
    assert obj.base_method(10) == 100
    assert obj.sub_method(10) == 50
"#
        );
    });
}

#[test]
fn mutation_fails() {
    Python::attach(|py| {
        let obj = Py::new(py, SubClass::new()).unwrap();
        let global = [("obj", obj)].into_py_dict(py).unwrap();
        let e = py
            .run(
                c"obj.base_set(lambda: obj.sub_set_and_ret(1))",
                Some(&global),
                None,
            )
            .unwrap_err();
        assert_eq!(&e.to_string(), "RuntimeError: Already borrowed");
    });
}

#[test]
fn is_subclass_and_is_instance() {
    Python::attach(|py| {
        let sub_ty = py.get_type::<SubClass>();
        let base_ty = py.get_type::<BaseClass>();
        assert!(sub_ty.is_subclass_of::<BaseClass>().unwrap());
        assert!(sub_ty.is_subclass(&base_ty).unwrap());

        let obj = Bound::new(py, SubClass::new()).unwrap().into_any();
        assert!(obj.is_instance_of::<SubClass>());
        assert!(obj.is_instance_of::<BaseClass>());
        assert!(obj.is_instance(&sub_ty).unwrap());
        assert!(obj.is_instance(&base_ty).unwrap());
    });
}

#[pyclass(subclass)]
struct BaseClassWithResult {
    _val: usize,
}

#[pymethods]
impl BaseClassWithResult {
    #[new]
    fn new(value: isize) -> PyResult<Self> {
        Ok(Self {
            _val: std::convert::TryFrom::try_from(value)?,
        })
    }
}

#[pyclass(extends=BaseClassWithResult)]
struct SubClass2 {}

#[pymethods]
impl SubClass2 {
    #[new]
    fn new(value: isize) -> PyResult<PyClassInitializer<Self>> {
        let base = BaseClassWithResult::new(value)?;
        Ok(PyClassInitializer::from(base).add_subclass(Self {}))
    }
}

#[test]
fn handle_result_in_new() {
    Python::attach(|py| {
        let subclass = py.get_type::<SubClass2>();
        py_run!(
            py,
            subclass,
            r#"
try:
    subclass(-10)
    assert False
except ValueError as e:
    pass
except Exception as e:
    raise e
"#
        );
    });
}

// Subclassing builtin types is not possible in the LIMITED API before 3.12
#[cfg(any(not(Py_LIMITED_API), Py_3_12))]
mod inheriting_native_type {
    use super::*;
    use pyo3::exceptions::PyException;

    #[cfg(not(GraalPy))]
    use {
        pyo3::types::{PyCapsule, PyDict},
        std::sync::{
            atomic::{AtomicBool, Ordering},
            Arc,
        },
    };

    #[cfg(not(any(PyPy, GraalPy)))]
    use pyo3::types::PySet;

    #[cfg(not(any(PyPy, GraalPy)))]
    #[pyclass(extends=PySet)]
    #[derive(Debug)]
    pub struct SetWithName {
        #[pyo3(get, name = "name")]
        _name: &'static str,
    }

    #[cfg(not(any(PyPy, GraalPy)))]
    #[pymethods]
    impl SetWithName {
        #[new]
        fn new() -> Self {
            SetWithName { _name: "Hello :)" }
        }
    }

    #[cfg(not(any(PyPy, GraalPy)))]
    #[test]
    fn inherit_set() {
        Python::attach(|py| {
            let set_sub = pyo3::Py::new(py, SetWithName::new()).unwrap();
            py_run!(
                py,
                set_sub,
                r#"set_sub.add(10); assert list(set_sub) == [10]; assert set_sub.name == "Hello :)""#
            );
        });
    }

    #[cfg(not(GraalPy))]
    #[pyclass(extends=PyDict)]
    #[derive(Debug)]
    struct DictWithName {
        #[pyo3(get, name = "name")]
        _name: &'static str,
    }

    #[cfg(not(GraalPy))]
    #[pymethods]
    impl DictWithName {
        #[new]
        fn new() -> Self {
            DictWithName { _name: "Hello :)" }
        }
    }

    #[cfg(not(GraalPy))]
    #[test]
    fn inherit_dict() {
        Python::attach(|py| {
            let dict_sub = pyo3::Py::new(py, DictWithName::new()).unwrap();
            py_run!(
                py,
                dict_sub,
                r#"dict_sub[0] = 1; assert dict_sub[0] == 1; assert dict_sub.name == "Hello :)""#
            );
        });
    }

    #[cfg(not(GraalPy))]
    #[test]
    fn inherit_dict_drop() {
        Python::attach(|py| {
            let dropped = Arc::new(AtomicBool::new(false));
            let destructor_drop = Arc::clone(&dropped);
            let item = PyCapsule::new_with_value_and_destructor(
                py,
                0,
                c"inherit_dict_drop",
                move |_, _| destructor_drop.store(true, Ordering::Relaxed),
            )
            .unwrap();

            let dict_sub = pyo3::Py::new(py, DictWithName::new()).unwrap();
            dict_sub.bind(py).set_item("foo", &item).unwrap();
            drop(item);
            assert!(!dropped.load(Ordering::Relaxed));
            drop(dict_sub);
            assert!(dropped.load(Ordering::Relaxed));
        })
    }

    #[pyclass(extends=PyException)]
    struct CustomException {
        #[pyo3(get)]
        context: &'static str,
    }

    #[pymethods]
    impl CustomException {
        #[new]
        fn new(_exc_arg: &Bound<'_, PyAny>) -> Self {
            CustomException {
                context: "Hello :)",
            }
        }
    }

    #[test]
    fn custom_exception() {
        Python::attach(|py| {
            let cls = py.get_type::<CustomException>();
            let dict = [("cls", &cls)].into_py_dict(py).unwrap();
            let res = py.run(
            c"e = cls('hello'); assert str(e) == 'hello'; assert e.context == 'Hello :)'; raise e",
            None,
            Some(&dict)
            );
            let err = res.unwrap_err();
            assert!(err.matches(py, &cls).unwrap(), "{}", err);

            // catching the exception in Python also works:
            py_run!(
                py,
                cls,
                r#"
                    try:
                        raise cls("foo")
                    except cls:
                        pass
                "#
            )
        })
    }

    #[cfg(Py_3_12)]
    #[pyclass(extends=pyo3::types::PyTzInfo)]
    struct TzInfoWithName {
        #[pyo3(get)]
        name: &'static str,
    }

    #[cfg(Py_3_12)]
    #[pymethods]
    impl TzInfoWithName {
        #[new]
        fn new() -> Self {
            Self { name: "Hello :)" }
        }

        #[pyo3(signature = (_dt, /))]
        fn utcoffset<'py>(
            &self,
            _dt: Option<&Bound<'_, pyo3::types::PyDateTime>>,
            py: Python<'py>,
        ) -> PyResult<Bound<'py, pyo3::types::PyDelta>> {
            pyo3::types::PyDelta::new(py, 0, 3600, 0, true)
        }
    }

    #[cfg(Py_3_12)]
    #[test]
    fn inherit_tzinfo() {
        Python::attach(|py| {
            let tz = pyo3::Py::new(py, TzInfoWithName::new()).unwrap();
            py_run!(
                py,
                tz,
                r#"
                    import datetime

                    assert isinstance(tz, datetime.tzinfo)
                    assert tz.name == "Hello :)"

                    dt = datetime.datetime(2024, 1, 1, tzinfo=tz)
                    assert dt.utcoffset() == datetime.timedelta(hours=1)
                "#
            );
        });
    }

    #[cfg(Py_3_12)]
    #[pyclass(extends=pyo3::types::PyList, subclass)]
    struct ListWithName {
        #[pyo3(get)]
        name: &'static str,
    }

    #[cfg(Py_3_12)]
    #[pymethods]
    impl ListWithName {
        #[new]
        fn new() -> Self {
            Self { name: "Hello :)" }
        }
    }

    #[cfg(Py_3_12)]
    #[pyclass(extends=ListWithName)]
    struct SubListWithName {
        #[pyo3(get)]
        sub_name: &'static str,
    }

    #[cfg(Py_3_12)]
    #[pymethods]
    impl SubListWithName {
        #[new]
        fn new() -> PyClassInitializer<Self> {
            PyClassInitializer::from(ListWithName::new()).add_subclass(Self {
                sub_name: "Sublist",
            })
        }
    }

    #[cfg(Py_3_12)]
    #[test]
    fn inherit_list() {
        Python::attach(|py| {
            let list_with_name = pyo3::Bound::new(py, ListWithName::new()).unwrap();
            let sub_list_with_name = pyo3::Bound::new(py, SubListWithName::new()).unwrap();
            py_run!(
                py,
                list_with_name sub_list_with_name,
                r#"
                    list_with_name.append(1)
                    assert list_with_name[0] == 1
                    assert list_with_name.name == "Hello :)", list_with_name.name

                    sub_list_with_name.append(1)
                    assert sub_list_with_name[0] == 1
                    assert sub_list_with_name.name == "Hello :)", sub_list_with_name.name
                    assert sub_list_with_name.sub_name == "Sublist", sub_list_with_name.sub_name
                "#
            );
        });
    }

    // Refcount tests for native type classes
    #[cfg(not(any(PyPy, GraalPy)))]
    #[test]
    fn test_setwitname_ref_counts() {
        assert_type_refcount_stable!(SetWithName);
    }

    #[cfg(not(GraalPy))]
    #[test]
    fn test_dictwithname_ref_counts() {
        assert_type_refcount_stable!(DictWithName);
    }

    #[test]
    fn test_customexception_ref_counts() {
        assert_type_refcount_stable!(CustomException, "custom_exception", r#"Type('test')"#);
    }

    #[cfg(Py_3_12)]
    #[test]
    fn test_tzinfowithname_ref_counts() {
        assert_type_refcount_stable!(TzInfoWithName);
    }

    #[cfg(Py_3_12)]
    #[test]
    fn test_listwithname_ref_counts() {
        assert_type_refcount_stable!(ListWithName);
    }

    #[cfg(Py_3_12)]
    #[test]
    fn test_sublistwithname_ref_counts() {
        assert_type_refcount_stable!(SubListWithName);
    }
}

#[pyclass(subclass)]
struct SimpleClass {}

#[pymethods]
impl SimpleClass {
    #[new]
    fn new() -> Self {
        Self {}
    }
}

// Generate refcount tests for all top-level types
#[test]
fn test_baseclass_ref_counts() {
    assert_type_refcount_stable!(BaseClass);
}

#[test]
fn test_subclass_ref_counts() {
    assert_type_refcount_stable!(SubClass);
}

#[test]
fn test_base_class_with_result_ref_counts() {
    assert_type_refcount_stable!(BaseClassWithResult, "base_class_with_result", "Type(10)");
}

#[test]
fn test_subclass2_ref_counts() {
    assert_type_refcount_stable!(SubClass2, "subclass2", "Type(10)");
}

#[test]
fn test_simpleclass_ref_counts() {
    assert_type_refcount_stable!(SimpleClass);
}
