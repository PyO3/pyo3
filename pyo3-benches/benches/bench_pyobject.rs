use codspeed_criterion_compat::{criterion_group, criterion_main, Bencher, Criterion};

use pyo3::prelude::*;

fn drop_many_objects(b: &mut Bencher<'_>) {
    Python::with_gil(|py| {
        b.iter(|| {
            for _ in 0..1000 {
                std::mem::drop(py.None());
            }
        });
    });
}

fn deferred_drop_many_objects(b: &mut Bencher<'_>) {
    b.iter_batched_ref(
        || Python::with_gil(|py| (0..1000).map(|_| py.None()).collect::<Vec<_>>()),
        |objects| {
            objects.clear();
            // Trigger deferred drops
            Python::with_gil(|_| {})
        },
        criterion::BatchSize::PerIteration,
    );
}

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("drop_many_objects", drop_many_objects);
    c.bench_function("deferred_drop_many_objects", deferred_drop_many_objects);
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
