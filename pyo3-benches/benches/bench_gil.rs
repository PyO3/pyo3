use codspeed_criterion_compat::{criterion_group, criterion_main, BatchSize, Bencher, Criterion};

use pyo3::{prelude::*, GILPool};

fn bench_clean_gilpool_new(b: &mut Bencher<'_>) {
    Python::with_gil(|_py| {
        b.iter(|| {
            let _ = unsafe { GILPool::new() };
        });
    });
}

fn bench_clean_acquire_gil(b: &mut Bencher<'_>) {
    // Acquiring first GIL will also create a "clean" GILPool, so this measures the Python overhead.
    b.iter(|| Python::with_gil(|_| {}));
}

fn bench_dirty_acquire_gil(b: &mut Bencher<'_>) {
    let obj = Python::with_gil(|py| py.None());
    b.iter_batched(
        || {
            // Clone and drop an object so that the GILPool has work to do.
            let _ = obj.clone();
        },
        |_| Python::with_gil(|_| {}),
        BatchSize::NumBatches(1),
    );
}

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("clean_gilpool_new", bench_clean_gilpool_new);
    c.bench_function("clean_acquire_gil", bench_clean_acquire_gil);
    c.bench_function("dirty_acquire_gil", bench_dirty_acquire_gil);
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
