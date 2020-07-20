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
fn test_unsendable_dict() {
    let gil = Python::acquire_gil();
    let py = gil.python();
    let inst = Py::new(py, UnsendableDictClass {}).unwrap();
    py_run!(py, inst, "assert inst.__dict__ == {}");
}
