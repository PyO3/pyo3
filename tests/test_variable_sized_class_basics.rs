#![cfg(all(Py_3_12, feature = "macros"))]

use pyo3::types::{PyDict, PyInt, PyTuple};
use pyo3::{prelude::*, types::PyType};
use pyo3::{py_run, PyTypeInfo};
use static_assertions::const_assert;

#[path = "../src/tests/common.rs"]
mod common;

#[pyclass(extends=PyType)]
#[derive(Default)]
struct ClassWithObjectField {
    #[pyo3(get, set)]
    value: Option<PyObject>,
}

#[pymethods]
impl ClassWithObjectField {
    #[pyo3(signature = (*_args, **_kwargs))]
    fn __init__(
        _slf: Bound<'_, ClassWithObjectField>,
        _args: Bound<'_, PyTuple>,
        _kwargs: Option<Bound<'_, PyDict>>,
    ) {
    }
}

#[test]
fn class_with_object_field() {
    Python::with_gil(|py| {
        let ty = py.get_type::<ClassWithObjectField>();
        const_assert!(<ClassWithObjectField as PyTypeInfo>::OPAQUE);
        py_run!(
            py,
            ty,
            "x = ty('X', (), {}); x.value = 5; assert x.value == 5"
        );
        py_run!(
            py,
            ty,
            "x = ty('X', (), {}); x.value = None; assert x.value == None"
        );

        let obj = Bound::new(py, ClassWithObjectField { value: None }).unwrap();
        py_run!(py, obj, "obj.value = 5");
        let obj_ref = obj.borrow();
        let value = obj_ref.value.as_ref().unwrap();
        assert_eq!(*value.downcast_bound::<PyInt>(py).unwrap(), 5);
    });
}
