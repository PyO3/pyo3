#[cfg(feature = "macros")]
use criterion::{criterion_group, criterion_main, Criterion};

#[cfg(feature = "macros")]
mod m {
    use pyo3::{class::PyObjectProtocol, prelude::*, type_object::LazyStaticType};

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

        fn __call__(&mut self, new_element: i32) -> usize {
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

    pub fn first_time_init(b: &mut criterion::Bencher) {
        let gil = Python::acquire_gil();
        let py = gil.python();
        b.iter(|| {
            // This is using an undocumented internal PyO3 API to measure pyclass performance; please
            // don't use this in your own code!
            let ty = LazyStaticType::new();
            ty.get_or_init::<MyClass>(py);
        });
    }
}

#[cfg(feature = "macros")]
fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("first_time_init", m::first_time_init);
}

#[cfg(feature = "macros")]
criterion_group!(benches, criterion_benchmark);

#[cfg(feature = "macros")]
criterion_main!(benches);

#[cfg(not(feature = "macros"))]
fn main() {
    unimplemented!(
        "benchmarking `bench_pyclass` is only available with the `macros` feature enabled"
    );
}
