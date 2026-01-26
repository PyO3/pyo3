#![cfg(feature = "macros")]

use pyo3::prelude::*;

#[pyclass(subclass)]
struct Base {
    num: u32,
}

#[pymethods]
impl Base {
    #[new]
    fn new() -> Self {
        Self { num: 0 }
    }

    fn __init__(&mut self) {
        self.num += 42
    }
}

#[test]
fn test_base_init() {
    Python::attach(|py| {
        let typeobj = py.get_type::<Base>();
        let obj = typeobj.call((), None).unwrap().cast_into::<Base>().unwrap();
        // check __init__ was called
        assert_eq!(obj.borrow().num, 42);
    });
}

#[pyclass(extends=Base)]
struct SubWithoutInit;

#[pymethods]
impl SubWithoutInit {
    #[new]
    fn new() -> (Self, Base) {
        (Self, Base::new())
    }
}

#[test]
fn test_subclass_without_init_calls_base_init() {
    Python::attach(|py| {
        let typeobj = py.get_type::<SubWithoutInit>();
        let obj = typeobj
            .call((), None)
            .unwrap()
            .cast_into::<SubWithoutInit>()
            .unwrap();
        // check Base.__init__ was called
        assert_eq!(obj.as_super().borrow().num, 42);
    });
}

#[pyclass(extends=Base)]
struct SubWithInit;

#[pymethods]
impl SubWithInit {
    #[new]
    fn new() -> (Self, Base) {
        (Self, Base::new())
    }

    fn __init__(mut slf: pyo3::PyClassGuardMut<'_, Self>) {
        slf.as_super().__init__(); // need to call super __init__ manually
        slf.as_super().num += 1;
    }
}

#[test]
fn test_subclass_with_init() {
    Python::attach(|py| {
        let typeobj = py.get_type::<SubWithInit>();
        let obj = typeobj
            .call((), None)
            .unwrap()
            .cast_into::<SubWithInit>()
            .unwrap();
        // check SubWithInit.__init__ was called, and Base.__init__ was only called once (through
        // SubWithInit.__init__)
        assert_eq!(obj.as_super().borrow().num, 43);
    });
}
