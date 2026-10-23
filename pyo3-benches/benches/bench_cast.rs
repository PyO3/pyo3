use std::hint::black_box;

use codspeed_criterion_compat::{criterion_group, criterion_main, BatchSize, Bencher, Criterion};

use pyo3::{
    prelude::*,
    types::{PyDict, PyList},
};

#[pyclass]
struct MyClass;

fn bench_cast_list(b: &mut Bencher<'_>) {
    Python::attach(|py| {
        let obj = PyDict::new(py).into_any();
        b.iter(|| black_box(&obj).cast::<PyList>().is_ok());
    });
}

fn bench_cast_or_opt_list(b: &mut Bencher<'_>) {
    Python::attach(|py| {
        let obj = PyDict::new(py).into_any();
        b.iter(|| black_box(&obj).cast_or_opt::<PyList>().is_some());
    });
}

fn bench_cast_pyclass(b: &mut Bencher<'_>) {
    Python::attach(|py| {
        let obj = PyDict::new(py).into_any();
        b.iter(|| black_box(&obj).cast::<MyClass>().is_ok());
    });
}

fn bench_cast_or_opt_pyclass(b: &mut Bencher<'_>) {
    Python::attach(|py| {
        let obj = PyDict::new(py).into_any();
        b.iter(|| black_box(&obj).cast_or_opt::<MyClass>().is_some());
    });
}

fn bench_cast_exact_pyclass(b: &mut Bencher<'_>) {
    Python::attach(|py| {
        let obj = PyDict::new(py).into_any();
        b.iter(|| black_box(&obj).cast_exact::<MyClass>().is_ok());
    });
}

fn bench_cast_exact_or_opt_pyclass(b: &mut Bencher<'_>) {
    Python::attach(|py| {
        let obj = PyDict::new(py).into_any();
        b.iter(|| black_box(&obj).cast_exact_or_opt::<MyClass>().is_some());
    });
}

fn bench_cast_into_pyclass(b: &mut Bencher<'_>) {
    Python::attach(|py| {
        let obj = PyDict::new(py).into_any();
        b.iter_batched(
            || obj.clone(),
            |obj| obj.cast_into::<MyClass>().is_ok(),
            BatchSize::SmallInput,
        );
    });
}

fn bench_cast_into_or_err_pyclass(b: &mut Bencher<'_>) {
    Python::attach(|py| {
        let obj = PyDict::new(py).into_any();
        b.iter_batched(
            || obj.clone(),
            |obj| obj.cast_into_or_err::<MyClass>().is_ok(),
            BatchSize::SmallInput,
        );
    });
}

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("cast_list", bench_cast_list);
    c.bench_function("cast_or_opt_list", bench_cast_or_opt_list);
    c.bench_function("cast_pyclass", bench_cast_pyclass);
    c.bench_function("cast_or_opt_pyclass", bench_cast_or_opt_pyclass);
    c.bench_function("cast_exact_pyclass", bench_cast_exact_pyclass);
    c.bench_function("cast_exact_or_opt_pyclass", bench_cast_exact_or_opt_pyclass);
    c.bench_function("cast_into_pyclass", bench_cast_into_pyclass);
    c.bench_function("cast_into_or_err_pyclass", bench_cast_into_or_err_pyclass);
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
