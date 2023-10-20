use codspeed_criterion_compat::{criterion_group, criterion_main, Bencher, Criterion};

use pyo3::{exceptions::PyValueError, prelude::*};

fn err_new_restore_and_fetch(b: &mut Bencher<'_>) {
    Python::with_gil(|py| {
        b.iter(|| {
            PyValueError::new_err("some exception message").restore(py);
            PyErr::fetch(py)
        })
    })
}

fn err_new_without_gil(b: &mut Bencher<'_>) {
    b.iter(|| PyValueError::new_err("some exception message"))
}

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("err_new_restore_and_fetch", err_new_restore_and_fetch);
    c.bench_function("err_new_without_gil", err_new_without_gil);
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
