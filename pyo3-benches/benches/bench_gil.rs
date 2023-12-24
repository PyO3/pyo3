use codspeed_criterion_compat::{
    black_box, criterion_group, criterion_main, BatchSize, Bencher, Criterion,
};

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
    let obj: PyObject = Python::with_gil(|py| py.None().into());
    b.iter_batched(
        || {
            // Clone and drop an object so that the GILPool has work to do.
            let _ = obj.clone();
        },
        |_| Python::with_gil(|_| {}),
        BatchSize::NumBatches(1),
    );
}

fn bench_allow_threads(b: &mut Bencher<'_>) {
    Python::with_gil(|py| {
        py.allow_threads().with(|| ());
        b.iter(|| py.allow_threads().with(|| black_box(42)));
    });
}

fn bench_local_allow_threads(b: &mut Bencher<'_>) {
    Python::with_gil(|py| {
        b.iter(|| unsafe { py.allow_threads().local() }.with(|| black_box(42)));
    });
}

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("clean_gilpool_new", bench_clean_gilpool_new);
    c.bench_function("clean_acquire_gil", bench_clean_acquire_gil);
    c.bench_function("dirty_acquire_gil", bench_dirty_acquire_gil);
    c.bench_function("allow_threads", bench_allow_threads);
    c.bench_function("local_allow_threads", bench_local_allow_threads);
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
