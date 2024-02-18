#![cfg(feature = "macros")]

use pyo3::prelude::*;
use pyo3::types::PyType;
use pyo3::{py_run, PyClass};

#[path = "../src/tests/common.rs"]
mod common;

#[pyclass]
struct EmptyClass {}

#[test]
fn empty_class() {
    Python::with_gil(|py| {
        let typeobj = py.get_type::<EmptyClass>();
        // By default, don't allow creating instances from python.
        assert!(typeobj.call((), None).is_err());

        py_assert!(py, typeobj, "typeobj.__name__ == 'EmptyClass'");
    });
}

#[pyclass]
struct UnitClass;

#[test]
fn unit_class() {
    Python::with_gil(|py| {
        let typeobj = py.get_type::<UnitClass>();
        // By default, don't allow creating instances from python.
        assert!(typeobj.call((), None).is_err());

        py_assert!(py, typeobj, "typeobj.__name__ == 'UnitClass'");
    });
}

/// Line1
///Line2
///  Line3
// this is not doc string
#[pyclass]
struct ClassWithDocs {
    /// Property field
    #[pyo3(get, set)]
    value: i32,

    /// Read-only property field
    #[pyo3(get)]
    readonly: i32,

    /// Write-only property field
    #[pyo3(set)]
    #[allow(dead_code)] // Rust detects field is never read
    writeonly: i32,
}

#[test]
fn class_with_docstr() {
    Python::with_gil(|py| {
        let typeobj = py.get_type::<ClassWithDocs>();
        py_run!(
            py,
            typeobj,
            "assert typeobj.__doc__ == 'Line1\\nLine2\\n Line3'"
        );
        py_run!(
            py,
            typeobj,
            "assert typeobj.value.__doc__ == 'Property field'"
        );
        py_run!(
            py,
            typeobj,
            "assert typeobj.readonly.__doc__ == 'Read-only property field'"
        );
        py_run!(
            py,
            typeobj,
            "assert typeobj.writeonly.__doc__ == 'Write-only property field'"
        );
    });
}

#[pyclass(name = "CustomName")]
struct EmptyClass2 {}

#[pymethods]
impl EmptyClass2 {
    #[pyo3(name = "custom_fn")]
    fn bar(&self) {}

    #[staticmethod]
    #[pyo3(name = "custom_static")]
    fn bar_static() {}

    #[getter]
    #[pyo3(name = "custom_getter")]
    fn foo(&self) -> i32 {
        5
    }
}

#[test]
fn custom_names() {
    Python::with_gil(|py| {
        let typeobj = py.get_type::<EmptyClass2>();
        py_assert!(py, typeobj, "typeobj.__name__ == 'CustomName'");
        py_assert!(py, typeobj, "typeobj.custom_fn.__name__ == 'custom_fn'");
        py_assert!(
            py,
            typeobj,
            "typeobj.custom_static.__name__ == 'custom_static'"
        );
        py_assert!(
            py,
            typeobj,
            "typeobj.custom_getter.__name__ == 'custom_getter'"
        );
        py_assert!(py, typeobj, "not hasattr(typeobj, 'bar')");
        py_assert!(py, typeobj, "not hasattr(typeobj, 'bar_static')");
        py_assert!(py, typeobj, "not hasattr(typeobj, 'foo')");
    });
}

#[pyclass]
struct RawIdents {
    #[pyo3(get, set)]
    r#type: i64,
}

#[pymethods]
impl RawIdents {
    fn r#fn(&self) {}
}

#[test]
fn test_raw_idents() {
    Python::with_gil(|py| {
        let typeobj = py.get_type::<RawIdents>();
        py_assert!(py, typeobj, "not hasattr(typeobj, 'r#fn')");
        py_assert!(py, typeobj, "hasattr(typeobj, 'fn')");
        py_assert!(py, typeobj, "hasattr(typeobj, 'type')");
    });
}

#[pyclass]
struct EmptyClassInModule {}

// Ignored because heap types do not show up as being in builtins, instead they
// raise AttributeError:
// https://github.com/python/cpython/blob/v3.11.1/Objects/typeobject.c#L541-L570
#[test]
#[ignore]
fn empty_class_in_module() {
    Python::with_gil(|py| {
        let module = PyModule::new(py, "test_module.nested").unwrap();
        module.add_class::<EmptyClassInModule>().unwrap();

        let ty = module.getattr("EmptyClassInModule").unwrap();
        assert_eq!(
            ty.getattr("__name__").unwrap().extract::<String>().unwrap(),
            "EmptyClassInModule"
        );

        let module: String = ty.getattr("__module__").unwrap().extract().unwrap();

        // Rationale: The class can be added to many modules, but will only be initialized once.
        // We currently have no way of determining a canonical module, so builtins is better
        // than using whatever calls init first.
        assert_eq!(module, "builtins");
    });
}

#[pyclass]
struct ClassWithObjectField {
    // It used to be that PyObject was not supported with (get, set)
    // - this test is just ensuring it compiles.
    #[pyo3(get, set)]
    value: PyObject,
}

#[pymethods]
impl ClassWithObjectField {
    #[new]
    fn new(value: PyObject) -> ClassWithObjectField {
        Self { value }
    }
}

#[test]
fn class_with_object_field() {
    Python::with_gil(|py| {
        let ty = py.get_type::<ClassWithObjectField>();
        py_assert!(py, ty, "ty(5).value == 5");
        py_assert!(py, ty, "ty(None).value == None");
    });
}

#[pyclass(unsendable, subclass)]
struct UnsendableBase {
    value: std::rc::Rc<usize>,
}

#[pymethods]
impl UnsendableBase {
    #[new]
    fn new(value: usize) -> UnsendableBase {
        Self {
            value: std::rc::Rc::new(value),
        }
    }

    #[getter]
    fn value(&self) -> usize {
        *self.value
    }
}

#[pyclass(extends=UnsendableBase)]
struct UnsendableChild {}

#[pymethods]
impl UnsendableChild {
    #[new]
    fn new(value: usize) -> (UnsendableChild, UnsendableBase) {
        (UnsendableChild {}, UnsendableBase::new(value))
    }
}

fn test_unsendable<T: PyClass + 'static>() -> PyResult<()> {
    let obj = Python::with_gil(|py| -> PyResult<_> {
        let obj: Py<T> = PyType::new_bound::<T>(py).call1((5,))?.extract()?;

        // Accessing the value inside this thread should not panic
        let caught_panic =
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| -> PyResult<_> {
                assert_eq!(obj.getattr(py, "value")?.extract::<usize>(py)?, 5);
                Ok(())
            }))
            .is_err();

        assert!(!caught_panic);
        Ok(obj)
    })?;

    let keep_obj_here = obj.clone();

    let caught_panic = std::thread::spawn(move || {
        // This access must panic
        Python::with_gil(|py| {
            obj.borrow(py);
        });
    })
    .join();

    Python::with_gil(|_py| drop(keep_obj_here));

    if let Err(err) = caught_panic {
        if let Some(msg) = err.downcast_ref::<String>() {
            panic!("{}", msg);
        }
    }

    Ok(())
}

/// If a class is marked as `unsendable`, it panics when accessed by another thread.
#[test]
#[cfg_attr(target_arch = "wasm32", ignore)]
#[should_panic(
    expected = "test_class_basics::UnsendableBase is unsendable, but sent to another thread"
)]
fn panic_unsendable_base() {
    test_unsendable::<UnsendableBase>().unwrap();
}

#[test]
#[cfg_attr(target_arch = "wasm32", ignore)]
#[should_panic(
    expected = "test_class_basics::UnsendableBase is unsendable, but sent to another thread"
)]
fn panic_unsendable_child() {
    test_unsendable::<UnsendableChild>().unwrap();
}

fn get_length(obj: &Bound<'_, PyAny>) -> PyResult<usize> {
    let length = obj.len()?;

    Ok(length)
}

#[pyclass]
struct ClassWithFromPyWithMethods {}

#[pymethods]
impl ClassWithFromPyWithMethods {
    fn instance_method(&self, #[pyo3(from_py_with = "get_length")] argument: usize) -> usize {
        argument
    }
    #[classmethod]
    fn classmethod(
        _cls: &Bound<'_, PyType>,
        #[pyo3(from_py_with = "Bound::<'_, PyAny>::len")] argument: usize,
    ) -> usize {
        argument
    }

    #[classmethod]
    fn classmethod_gil_ref(
        _cls: &PyType,
        #[pyo3(from_py_with = "PyAny::len")] argument: usize,
    ) -> usize {
        argument
    }

    #[staticmethod]
    fn staticmethod(#[pyo3(from_py_with = "get_length")] argument: usize) -> usize {
        argument
    }
}

#[test]
fn test_pymethods_from_py_with() {
    Python::with_gil(|py| {
        let instance = Py::new(py, ClassWithFromPyWithMethods {}).unwrap();

        py_run!(
            py,
            instance,
            r#"
        arg = {1: 1, 2: 3}

        assert instance.instance_method(arg) == 2
        assert instance.classmethod(arg) == 2
        assert instance.classmethod_gil_ref(arg) == 2
        assert instance.staticmethod(arg) == 2
        "#
        );
    })
}

#[pyclass]
struct TupleClass(#[pyo3(get, set, name = "value")] i32);

#[test]
fn test_tuple_struct_class() {
    Python::with_gil(|py| {
        let typeobj = py.get_type::<TupleClass>();
        assert!(typeobj.call((), None).is_err());

        py_assert!(py, typeobj, "typeobj.__name__ == 'TupleClass'");

        let instance = Py::new(py, TupleClass(5)).unwrap();
        py_run!(
            py,
            instance,
            r#"
        assert instance.value == 5;
        instance.value = 1234;
        assert instance.value == 1234;
        "#
        );

        assert_eq!(instance.borrow(py).0, 1234);
    });
}

#[pyclass(dict, subclass)]
struct DunderDictSupport {
    // Make sure that dict_offset runs with non-zero sized Self
    _pad: [u8; 32],
}

#[test]
#[cfg_attr(all(Py_LIMITED_API, not(Py_3_9)), ignore)]
fn dunder_dict_support() {
    Python::with_gil(|py| {
        let inst = PyCell::new(
            py,
            DunderDictSupport {
                _pad: *b"DEADBEEFDEADBEEFDEADBEEFDEADBEEF",
            },
        )
        .unwrap();
        py_run!(
            py,
            inst,
            r#"
        inst.a = 1
        assert inst.a == 1
    "#
        );
    });
}

// Accessing inst.__dict__ only supported in limited API from Python 3.10
#[test]
#[cfg_attr(all(Py_LIMITED_API, not(Py_3_10)), ignore)]
fn access_dunder_dict() {
    Python::with_gil(|py| {
        let inst = PyCell::new(
            py,
            DunderDictSupport {
                _pad: *b"DEADBEEFDEADBEEFDEADBEEFDEADBEEF",
            },
        )
        .unwrap();
        py_run!(
            py,
            inst,
            r#"
        inst.a = 1
        assert inst.__dict__ == {'a': 1}
    "#
        );
    });
}

// If the base class has dict support, child class also has dict
#[pyclass(extends=DunderDictSupport)]
struct InheritDict {
    _value: usize,
}

#[test]
#[cfg_attr(all(Py_LIMITED_API, not(Py_3_9)), ignore)]
fn inherited_dict() {
    Python::with_gil(|py| {
        let inst = PyCell::new(
            py,
            (
                InheritDict { _value: 0 },
                DunderDictSupport {
                    _pad: *b"DEADBEEFDEADBEEFDEADBEEFDEADBEEF",
                },
            ),
        )
        .unwrap();
        py_run!(
            py,
            inst,
            r#"
        inst.a = 1
        assert inst.a == 1
    "#
        );
    });
}

#[pyclass(weakref, dict)]
struct WeakRefDunderDictSupport {
    // Make sure that weaklist_offset runs with non-zero sized Self
    _pad: [u8; 32],
}

#[test]
#[cfg_attr(all(Py_LIMITED_API, not(Py_3_9)), ignore)]
fn weakref_dunder_dict_support() {
    Python::with_gil(|py| {
        let inst = PyCell::new(
            py,
            WeakRefDunderDictSupport {
                _pad: *b"DEADBEEFDEADBEEFDEADBEEFDEADBEEF",
            },
        )
        .unwrap();
        py_run!(
            py,
            inst,
            "import weakref; assert weakref.ref(inst)() is inst; inst.a = 1; assert inst.a == 1"
        );
    });
}

#[pyclass(weakref, subclass)]
struct WeakRefSupport {
    _pad: [u8; 32],
}

#[test]
#[cfg_attr(all(Py_LIMITED_API, not(Py_3_9)), ignore)]
fn weakref_support() {
    Python::with_gil(|py| {
        let inst = PyCell::new(
            py,
            WeakRefSupport {
                _pad: *b"DEADBEEFDEADBEEFDEADBEEFDEADBEEF",
            },
        )
        .unwrap();
        py_run!(
            py,
            inst,
            "import weakref; assert weakref.ref(inst)() is inst"
        );
    });
}

// If the base class has weakref support, child class also has weakref.
#[pyclass(extends=WeakRefSupport)]
struct InheritWeakRef {
    _value: usize,
}

#[test]
#[cfg_attr(all(Py_LIMITED_API, not(Py_3_9)), ignore)]
fn inherited_weakref() {
    Python::with_gil(|py| {
        let inst = PyCell::new(
            py,
            (
                InheritWeakRef { _value: 0 },
                WeakRefSupport {
                    _pad: *b"DEADBEEFDEADBEEFDEADBEEFDEADBEEF",
                },
            ),
        )
        .unwrap();
        py_run!(
            py,
            inst,
            "import weakref; assert weakref.ref(inst)() is inst"
        );
    });
}

#[test]
fn access_frozen_class_without_gil() {
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[pyclass(frozen)]
    struct FrozenCounter {
        value: AtomicUsize,
    }

    let py_counter: Py<FrozenCounter> = Python::with_gil(|py| {
        let counter = FrozenCounter {
            value: AtomicUsize::new(0),
        };

        let cell = PyCell::new(py, counter).unwrap();

        cell.get().value.fetch_add(1, Ordering::Relaxed);

        cell.into()
    });

    assert_eq!(py_counter.get().value.load(Ordering::Relaxed), 1);
}

#[test]
#[cfg(Py_3_8)] // sys.unraisablehook not available until Python 3.8
#[cfg_attr(target_arch = "wasm32", ignore)]
fn drop_unsendable_elsewhere() {
    use common::UnraisableCapture;
    use std::sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    };
    use std::thread::spawn;

    #[pyclass(unsendable)]
    struct Unsendable {
        dropped: Arc<AtomicBool>,
    }

    impl Drop for Unsendable {
        fn drop(&mut self) {
            self.dropped.store(true, Ordering::SeqCst);
        }
    }

    Python::with_gil(|py| {
        let capture = UnraisableCapture::install(py);

        let dropped = Arc::new(AtomicBool::new(false));

        let unsendable = Py::new(
            py,
            Unsendable {
                dropped: dropped.clone(),
            },
        )
        .unwrap();

        py.allow_threads(|| {
            spawn(move || {
                Python::with_gil(move |_py| {
                    drop(unsendable);
                });
            })
            .join()
            .unwrap();
        });

        assert!(!dropped.load(Ordering::SeqCst));

        let (err, object) = capture.borrow_mut(py).capture.take().unwrap();
        assert_eq!(err.to_string(), "RuntimeError: test_class_basics::drop_unsendable_elsewhere::Unsendable is unsendable, but is being dropped on another thread");
        assert!(object.is_none(py));

        capture.borrow_mut(py).uninstall(py);
    });
}
