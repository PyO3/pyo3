use codspeed_criterion_compat::{criterion_group, criterion_main, Bencher, Criterion};

use pyo3::prelude::*;

fn bench_clean_acquire_gil(b: &mut Bencher<'_>) {
    // Acquiring first GIL will also create a "clean" GILPool, so this measures the Python overhead.
    b.iter(|| Python::with_gil(|_| {}));
}

fn bench_dirty_acquire_gil(b: &mut Bencher<'_>) {
    let obj = Python::with_gil(|py| py.None());
    // Drop the returned clone of the object so that the reference pool has work to do.
    b.iter(|| Python::with_gil(|py| obj.clone_ref(py)));
}

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("clean_acquire_gil", bench_clean_acquire_gil);
    c.bench_function("dirty_acquire_gil", bench_dirty_acquire_gil);
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
