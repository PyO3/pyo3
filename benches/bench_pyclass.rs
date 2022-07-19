use criterion::{criterion_group, criterion_main, Criterion};
use pyo3::{prelude::*, type_object::LazyStaticType};

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

    /// A basic __str__ implementation.
    fn __str__(&self) -> &'static str {
        "MyClass"
    }
}

pub fn first_time_init(b: &mut criterion::Bencher<'_>) {
    Python::with_gil(|py| {
        b.iter(|| {
            // This is using an undocumented internal PyO3 API to measure pyclass performance; please
            // don't use this in your own code!
            let ty = LazyStaticType::new();
            ty.get_or_init::<MyClass>(py);
        });
    });
}

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("first_time_init", first_time_init);
}

criterion_group!(benches, criterion_benchmark);

criterion_main!(benches);
