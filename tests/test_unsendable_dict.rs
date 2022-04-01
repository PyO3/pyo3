#![cfg(feature = "macros")]

use pyo3::prelude::*;
use pyo3::py_run;

#[pyclass(dict, unsendable)]
struct UnsendableDictClass {}

#[pymethods]
impl UnsendableDictClass {
    #[new]
    fn new() -> Self {
        UnsendableDictClass {}
    }
}

#[test]
#[cfg_attr(all(Py_LIMITED_API, not(Py_3_10)), ignore)]
fn test_unsendable_dict() {
    let gil = Python::acquire_gil();
    let py = gil.python();
    let inst = Py::new(py, UnsendableDictClass {}).unwrap();
    py_run!(py, inst, "assert inst.__dict__ == {}");
}

#[pyclass(dict, unsendable, weakref)]
struct UnsendableDictClassWithWeakRef {}

#[pymethods]
impl UnsendableDictClassWithWeakRef {
    #[new]
    fn new() -> Self {
        Self {}
    }
}

#[test]
#[cfg_attr(all(Py_LIMITED_API, not(Py_3_10)), ignore)]
fn test_unsendable_dict_with_weakref() {
    let gil = Python::acquire_gil();
    let py = gil.python();
    let inst = Py::new(py, UnsendableDictClassWithWeakRef {}).unwrap();
    py_run!(py, inst, "assert inst.__dict__ == {}");
    py_run!(
        py,
        inst,
        "import weakref; assert weakref.ref(inst)() is inst; inst.a = 1; assert inst.a == 1"
    );
}
