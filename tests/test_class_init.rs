#![cfg(feature = "macros")]

use pyo3::{
    exceptions::PyValueError,
    prelude::*,
    types::{IntoPyDict, PyDict, PyTuple},
};

#[pyclass]
#[derive(Default)]
struct EmptyClassWithInit {}

#[pymethods]
impl EmptyClassWithInit {
    #[new]
    #[pyo3(signature = (*_args, **_kwargs))]
    fn new(_args: &Bound<'_, PyTuple>, _kwargs: Option<&Bound<'_, PyDict>>) -> Self {
        EmptyClassWithInit {}
    }

    fn __init__(&self) {}
}

#[test]
fn empty_class_with_init() {
    Python::with_gil(|py| {
        let typeobj = py.get_type::<EmptyClassWithInit>();
        assert!(typeobj
            .call((), None)
            .unwrap()
            .downcast::<EmptyClassWithInit>()
            .is_ok());

        // Calling with arbitrary args or kwargs is not ok
        assert!(typeobj.call(("some", "args"), None).is_err());
        assert!(typeobj
            .call((), Some(&[("some", "kwarg")].into_py_dict(py).unwrap()))
            .is_err());
    });
}

#[pyclass]
struct SimpleInit {
    pub number: u64,
}

impl Default for SimpleInit {
    fn default() -> Self {
        Self { number: 2 }
    }
}

#[pymethods]
impl SimpleInit {
    #[new]
    fn new() -> SimpleInit {
        SimpleInit { number: 1 }
    }

    fn __init__(&mut self) {
        assert_eq!(self.number, 2);
        self.number = 3;
    }
}

#[test]
fn simple_init() {
    Python::with_gil(|py| {
        let typeobj = py.get_type::<SimpleInit>();
        let obj = typeobj.call((), None).unwrap();
        let obj = obj.downcast::<SimpleInit>().unwrap();
        assert_eq!(obj.borrow().number, 3);

        // Calling with arbitrary args or kwargs is not ok
        assert!(typeobj.call(("some", "args"), None).is_err());
        assert!(typeobj
            .call((), Some(&[("some", "kwarg")].into_py_dict(py).unwrap()))
            .is_err());
    });
}

#[pyclass]
struct InitWithTwoArgs {
    data1: i32,
    data2: i32,
}

impl Default for InitWithTwoArgs {
    fn default() -> Self {
        Self {
            data1: 123,
            data2: 234,
        }
    }
}

#[pymethods]
impl InitWithTwoArgs {
    #[new]
    fn new(arg1: i32, _arg2: i32) -> Self {
        InitWithTwoArgs {
            data1: arg1,
            data2: 0,
        }
    }

    fn __init__(&mut self, _arg1: i32, arg2: i32) {
        assert_eq!(self.data1, 123);
        assert_eq!(self.data2, 234);
        self.data2 = arg2;
    }
}

#[test]
fn init_with_two_args() {
    Python::with_gil(|py| {
        let typeobj = py.get_type::<InitWithTwoArgs>();
        let wrp = typeobj
            .call((10, 20), None)
            .map_err(|e| e.display(py))
            .unwrap();
        let obj = wrp.downcast::<InitWithTwoArgs>().unwrap();
        let obj_ref = obj.borrow();
        assert_eq!(obj_ref.data1, 123);
        assert_eq!(obj_ref.data2, 20);

        assert!(typeobj.call(("a", "b", "c"), None).is_err());
    });
}

#[pyclass]
#[derive(Default)]
struct InitWithVarArgs {
    args: Option<String>,
    kwargs: Option<String>,
}

#[pymethods]
impl InitWithVarArgs {
    #[new]
    #[pyo3(signature = (*_args, **_kwargs))]
    fn new(_args: &Bound<'_, PyTuple>, _kwargs: Option<&Bound<'_, PyDict>>) -> Self {
        InitWithVarArgs {
            args: None,
            kwargs: None,
        }
    }

    #[pyo3(signature = (*args, **kwargs))]
    fn __init__(&mut self, args: &Bound<'_, PyTuple>, kwargs: Option<&Bound<'_, PyDict>>) {
        self.args = Some(args.to_string());
        self.kwargs = Some(kwargs.map(|kwargs| kwargs.to_string()).unwrap_or_default());
    }
}

#[test]
fn init_with_var_args() {
    Python::with_gil(|py| {
        let typeobj = py.get_type::<InitWithVarArgs>();
        let kwargs = [("a", 1), ("b", 42)].into_py_dict(py).unwrap();
        let wrp = typeobj
            .call((10, 20), Some(&kwargs))
            .map_err(|e| e.display(py))
            .unwrap();
        let obj = wrp.downcast::<InitWithVarArgs>().unwrap();
        let obj_ref = obj.borrow();
        assert_eq!(obj_ref.args, Some("(10, 20)".to_owned()));
        assert_eq!(obj_ref.kwargs, Some("{'a': 1, 'b': 42}".to_owned()));
    });
}

#[pyclass(subclass)]
struct SuperClass {
    #[pyo3(get)]
    rust_new: bool,
    #[pyo3(get)]
    rust_default: bool,
    #[pyo3(get)]
    rust_init: bool,
}

impl Default for SuperClass {
    fn default() -> Self {
        Self {
            rust_new: false,
            rust_default: true,
            rust_init: false,
        }
    }
}

#[pymethods]
impl SuperClass {
    #[new]
    fn new() -> Self {
        SuperClass {
            rust_new: true,
            rust_default: false,
            rust_init: false,
        }
    }

    fn __init__(&mut self) {
        assert!(!self.rust_new);
        assert!(self.rust_default);
        assert!(!self.rust_init);
        self.rust_init = true;
    }
}

#[test]
fn subclass_init() {
    Python::with_gil(|py| {
        let super_cls = py.get_type::<SuperClass>();
        let source = pyo3_ffi::c_str!(pyo3::indoc::indoc!(
            r#"
            class Class(SuperClass):
                pass
            c = Class()
            assert c.rust_new is False  # overridden because __init__ called
            assert c.rust_default is True
            assert c.rust_init is True

            class Class(SuperClass):
                def __init__(self):
                    self.py_init = True
            c = Class()
            assert c.rust_new is True  # not overridden because __init__ not called
            assert c.rust_default is False
            assert c.rust_init is False
            assert c.py_init is True

            class Class(SuperClass):
                def __init__(self):
                    super().__init__()
                    self.py_init = True
            c = Class()
            assert c.rust_new is False  # overridden because __init__ called
            assert c.rust_default is True
            assert c.rust_init is True
            assert c.py_init is True
            "#
        ));
        let globals = PyModule::import(py, "__main__").unwrap().dict();
        globals.set_item("SuperClass", super_cls).unwrap();
        py.run(source, Some(&globals), None)
            .map_err(|e| e.display(py))
            .unwrap();
    });
}

#[pyclass(extends=SuperClass)]
struct SubClass {
    #[pyo3(get)]
    rust_subclass_new: bool,
    #[pyo3(get)]
    rust_subclass_default: bool,
    #[pyo3(get)]
    rust_subclass_init: bool,
}

impl Default for SubClass {
    fn default() -> Self {
        Self {
            rust_subclass_new: false,
            rust_subclass_default: true,
            rust_subclass_init: false,
        }
    }
}

#[pymethods]
impl SubClass {
    #[new]
    fn new() -> (Self, SuperClass) {
        (
            SubClass {
                rust_subclass_new: true,
                rust_subclass_default: false,
                rust_subclass_init: false,
            },
            SuperClass::new(),
        )
    }

    fn __init__(&mut self) {
        assert!(!self.rust_subclass_new);
        assert!(self.rust_subclass_default);
        assert!(!self.rust_subclass_init);
        self.rust_subclass_init = true;
    }
}

#[test]
#[should_panic(expected = "initialize_with_default does not currently support multi-level inheritance")]
fn subclass_pyclass_init() {
    Python::with_gil(|py| {
        let sub_cls = py.get_type::<SubClass>();
        let source = pyo3_ffi::c_str!(pyo3::indoc::indoc!(
            r#"
            c = SubClass()
            assert c.rust_new is True
            assert c.rust_default is False
            assert c.rust_init is False
            assert c.rust_subclass_new is False  # overridden by calling __init__
            assert c.rust_subclass_default is True
            assert c.rust_subclass_init is True
            "#
        ));
        let globals = PyModule::import(py, "__main__").unwrap().dict();
        globals.set_item("SubClass", sub_cls).unwrap();
        py.run(source, Some(&globals), None)
            .map_err(|e| e.display(py))
            .unwrap();
    });
}

#[pyclass]
#[derive(Debug, Default)]
struct InitWithCustomError {}

struct CustomError;

impl From<CustomError> for PyErr {
    fn from(_error: CustomError) -> PyErr {
        PyValueError::new_err("custom error")
    }
}

#[pymethods]
impl InitWithCustomError {
    #[new]
    fn new(_should_raise: bool) -> InitWithCustomError {
        InitWithCustomError {}
    }

    fn __init__(&self, should_raise: bool) -> Result<(), CustomError> {
        if should_raise {
            Err(CustomError)
        } else {
            Ok(())
        }
    }
}

#[test]
fn init_with_custom_error() {
    Python::with_gil(|py| {
        let typeobj = py.get_type::<InitWithCustomError>();
        typeobj.call((false,), None).unwrap();
        let err = typeobj.call((true,), None).unwrap_err();
        assert_eq!(err.to_string(), "ValueError: custom error");
    });
}
