use pyo3::prelude::*;

#[pyclass]
struct TestClass {
    num: u32,
}

fn main() {
    let t = TestClass { num: 10 };

    let gil = Python::acquire_gil();
    let py = gil.python();

    let pyvalue = Py::new(py, t).unwrap().to_object(py);
    let t: TestClass = pyvalue.extract(py).unwrap();
}
