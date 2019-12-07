use pyo3::prelude::*;
use pyo3::py_run;
use pyo3::pyclass::PyClassShell;
use pyo3::types::PyAny;
use pyo3::FromPyPointer;

#[pyclass]
struct Class {
    member: i32,
}

#[pymethods]
impl Class {
    fn hello(&self) -> i32 {
        self.member
    }
}

#[test]
fn test_shell() {
    let class = Class { member: 128 };
    let gil = Python::acquire_gil();
    let py = gil.python();
    // let obj: &PyAny = unsafe { FromPyPointer::from_owned_ptr(py, PyClassShell::new(py, class)) };
    // py_run!(py, obj, "assert obj.hello() == 128");
}
