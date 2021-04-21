#![feature(test)]

extern crate test;
use pyo3::{class::PyObjectProtocol, prelude::*, type_object::LazyStaticType};
use test::Bencher;

/// This is a feature-rich class instance used to benchmark various parts of the pyclass lifecycle.
#[pyclass]
struct MyClass {
    #[pyo3(get, set)]
    elements: Vec<i32>,
}

#[pymethods]
impl MyClass {
    #[new]
    fn new(elements: Vec<i32>) -> Self {
        Self { elements }
    }

    #[call]
    fn call(&mut self, new_element: i32) -> usize {
        self.elements.push(new_element);
        self.elements.len()
    }
}

#[pyproto]
impl PyObjectProtocol for MyClass {
    /// A basic __str__ implementation.
    fn __str__(&self) -> &'static str {
        "MyClass"
    }
}

#[bench]
fn first_time_init(b: &mut Bencher) {
    let gil = Python::acquire_gil();
    let py = gil.python();
    b.iter(|| {
        // This is using an undocumented internal PyO3 API to measure pyclass performance; please
        // don't use this in your own code!
        let ty = LazyStaticType::new();
        ty.get_or_init::<MyClass>(py);
    });
}
