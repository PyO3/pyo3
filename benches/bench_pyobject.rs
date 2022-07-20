use criterion::{criterion_group, criterion_main, Bencher, Criterion};

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

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("drop_many_objects", drop_many_objects);
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
