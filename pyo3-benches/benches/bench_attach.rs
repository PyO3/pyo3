use codspeed_criterion_compat::{criterion_group, criterion_main, Bencher, Criterion};

use pyo3::prelude::*;

fn bench_clean_attach(b: &mut Bencher<'_>) {
    // Acquiring first GIL will also create a "clean" GILPool, so this measures the Python overhead.
    b.iter(|| Python::attach(|_| {}));
}

fn bench_dirty_attach(b: &mut Bencher<'_>) {
    let obj = Python::attach(|py| py.None());
    // Drop the returned clone of the object so that the reference pool has work to do.
    b.iter(|| Python::attach(|py| obj.clone_ref(py)));
}

fn bench_empty_pool_attach(b: &mut Bencher<'_>) {
    // Drop the returned clone while detached so that the reference pool exists but is empty.
    drop(Python::attach(|py| py.None()));
    b.iter(|| Python::attach(|_| {}));
}

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("clean_attach", bench_clean_attach);
    c.bench_function("dirty_attach", bench_dirty_attach);
    c.bench_function("empty_pool_attach", bench_empty_pool_attach);
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
