#![cfg(feature = "macros")]

use pyo3::prelude::*;
use pyo3::py_run;

use pyo3::ffi;
use pyo3::types::IntoPyDict;

mod test_utils;

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
            ffi::c_str!("class A(SubclassAble): pass\nassert issubclass(A, SubclassAble)"),
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
    fn new() -> (Self, BaseClass) {
        (SubClass { val2: 5 }, BaseClass { val1: 10 })
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
                ffi::c_str!("obj.base_set(lambda: obj.sub_set_and_ret(1))"),
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
    fn new(value: isize) -> PyResult<(Self, BaseClassWithResult)> {
        let base = BaseClassWithResult::new(value)?;
        Ok((Self {}, base))
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
    assert Fals
except ValueError as e:
    pass
except Exception as e:
    raise e
"#
        );
    });
}

// Subclassing builtin types is not allowed in the LIMITED API.
#[cfg(not(Py_LIMITED_API))]
mod inheriting_native_type {
    use super::*;
    use pyo3::exceptions::PyException;
    use pyo3::types::PyDict;

    #[cfg(not(any(PyPy, GraalPy)))]
    #[test]
    fn inherit_set() {
        use pyo3::types::PySet;

        #[pyclass(extends=PySet)]
        #[derive(Debug)]
        struct SetWithName {
            #[pyo3(get, name = "name")]
            _name: &'static str,
        }

        #[pymethods]
        impl SetWithName {
            #[new]
            fn new() -> Self {
                SetWithName { _name: "Hello :)" }
            }
        }

        Python::attach(|py| {
            let set_sub = pyo3::Py::new(py, SetWithName::new()).unwrap();
            py_run!(
                py,
                set_sub,
                r#"set_sub.add(10); assert list(set_sub) == [10]; assert set_sub.name == "Hello :)""#
            );
        });
    }

    #[pyclass(extends=PyDict)]
    #[derive(Debug)]
    struct DictWithName {
        #[pyo3(get, name = "name")]
        _name: &'static str,
    }

    #[pymethods]
    impl DictWithName {
        #[new]
        fn new() -> Self {
            DictWithName { _name: "Hello :)" }
        }
    }

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

    #[test]
    fn inherit_dict_drop() {
        Python::attach(|py| {
            let dict_sub = pyo3::Py::new(py, DictWithName::new()).unwrap();
            assert_eq!(dict_sub.get_refcnt(py), 1);

            let item = &py.eval(ffi::c_str!("object()"), None, None).unwrap();
            assert_eq!(item.get_refcnt(), 1);

            dict_sub.bind(py).set_item("foo", item).unwrap();
            assert_eq!(item.get_refcnt(), 2);

            drop(dict_sub);
            assert_eq!(item.get_refcnt(), 1);
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
            ffi::c_str!("e = cls('hello'); assert str(e) == 'hello'; assert e.context == 'Hello :)'; raise e"),
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

#[test]
fn test_subclass_ref_counts() {
    // regression test for issue #1363
    Python::attach(|py| {
        #[allow(non_snake_case)]
        let SimpleClass = py.get_type::<SimpleClass>();
        py_run!(
            py,
            SimpleClass,
            r#"
            import gc
            import sys

            class SubClass(SimpleClass):
                pass

            gc.collect()
            count = sys.getrefcount(SubClass)

            for i in range(1000):
                c = SubClass()
                del c

            gc.collect()
            after = sys.getrefcount(SubClass)
            # depending on Python's GC the count may be either identical or exactly 1000 higher,
            # both are expected values that are not representative of the issue.
            #
            # (With issue #1363 the count will be decreased.)
            assert after == count or (after == count + 1000), f"{after} vs {count}"
            "#
        );
    })
}
