use codspeed_criterion_compat::{criterion_group, criterion_main, BatchSize, Bencher, Criterion};
use pyo3::conversion::IntoPyObjectExt;
use pyo3::types::PyInt;
use pyo3::{impl_::pyclass::LazyTypeObject, prelude::*};

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

pub fn first_time_init(b: &mut Bencher<'_>) {
    Python::attach(|py| {
        b.iter(|| {
            // This is using an undocumented internal PyO3 API to measure pyclass performance; please
            // don't use this in your own code!
            let ty = LazyTypeObject::<MyClass>::new();
            ty.get_or_try_init(py).unwrap();
        });
    });
}

pub fn bench_pyclass(c: &mut Criterion) {
    c.bench_function("bench_pyclass_create", |b| {
        Python::attach(|py| {
            b.iter_batched(
                || vec![1, 2, 3],
                |elements| {
                    MyClass::new(elements).into_py_any(py).unwrap();
                },
                BatchSize::SmallInput,
            );
        });
    });
    c.bench_function("bench_call", |b| {
        Python::attach(|py| {
            b.iter_batched(
                || {
                    (
                        MyClass::new(vec![1, 2, 3]).into_py_any(py).unwrap(),
                        PyInt::new(py, 4),
                    )
                },
                |(inst, arg)| {
                    inst.call1(py, (arg,)).unwrap();
                },
                BatchSize::SmallInput,
            );
        });
    });
    c.bench_function("bench_str", |b| {
        Python::attach(|py| {
            let inst = MyClass::new(vec![1, 2, 3]).into_py_any(py).unwrap();
            let bound = inst.bind(py);
            b.iter(|| bound.str());
        });
    });
}

fn bench_first_time_init(c: &mut Criterion) {
    c.bench_function("first_time_init", first_time_init);
}

criterion_group!(benches, bench_first_time_init, bench_pyclass);

criterion_main!(benches);
