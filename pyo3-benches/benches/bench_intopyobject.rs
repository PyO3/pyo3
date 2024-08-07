use std::hint::black_box;

use codspeed_criterion_compat::{criterion_group, criterion_main, Bencher, Criterion};

use pyo3::conversion::IntoPyObject;
use pyo3::prelude::*;
use pyo3::types::PyBytes;

fn bench_bytes_new(b: &mut Bencher<'_>, data: &[u8]) {
    Python::with_gil(|py| {
        b.iter_with_large_drop(|| PyBytes::new(py, black_box(data)));
    });
}

fn bytes_new_small(b: &mut Bencher<'_>) {
    bench_bytes_new(b, &[]);
}

fn bytes_new_medium(b: &mut Bencher<'_>) {
    let data = (0..u8::MAX).into_iter().collect::<Vec<u8>>();
    bench_bytes_new(b, &data);
}

fn bytes_new_large(b: &mut Bencher<'_>) {
    let data = vec![10u8; 100_000];
    bench_bytes_new(b, &data);
}

fn bench_bytes_into_pyobject(b: &mut Bencher<'_>, data: &[u8]) {
    Python::with_gil(|py| {
        b.iter_with_large_drop(|| black_box(data).into_pyobject(py));
    });
}

fn byte_slice_into_pyobject_small(b: &mut Bencher<'_>) {
    bench_bytes_into_pyobject(b, &[]);
}

fn byte_slice_into_pyobject_medium(b: &mut Bencher<'_>) {
    let data = (0..u8::MAX).into_iter().collect::<Vec<u8>>();
    bench_bytes_into_pyobject(b, &data);
}

fn byte_slice_into_pyobject_large(b: &mut Bencher<'_>) {
    let data = vec![10u8; 100_000];
    bench_bytes_into_pyobject(b, &data);
}

fn byte_slice_into_py(b: &mut Bencher<'_>) {
    Python::with_gil(|py| {
        let data = (0..u8::MAX).into_iter().collect::<Vec<u8>>();
        let bytes = data.as_slice();
        b.iter_with_large_drop(|| black_box(bytes).into_py(py));
    });
}

fn vec_into_pyobject(b: &mut Bencher<'_>) {
    Python::with_gil(|py| {
        let bytes = (0..u8::MAX).into_iter().collect::<Vec<u8>>();
        b.iter_with_large_drop(|| black_box(&bytes).clone().into_pyobject(py));
    });
}

fn vec_into_py(b: &mut Bencher<'_>) {
    Python::with_gil(|py| {
        let bytes = (0..u8::MAX).into_iter().collect::<Vec<u8>>();
        b.iter_with_large_drop(|| black_box(&bytes).clone().into_py(py));
    });
}

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("bytes_new_small", bytes_new_small);
    c.bench_function("bytes_new_medium", bytes_new_medium);
    c.bench_function("bytes_new_large", bytes_new_large);
    c.bench_function(
        "byte_slice_into_pyobject_small",
        byte_slice_into_pyobject_small,
    );
    c.bench_function(
        "byte_slice_into_pyobject_medium",
        byte_slice_into_pyobject_medium,
    );
    c.bench_function(
        "byte_slice_into_pyobject_large",
        byte_slice_into_pyobject_large,
    );
    c.bench_function("byte_slice_into_py", byte_slice_into_py);
    c.bench_function("vec_into_pyobject", vec_into_pyobject);
    c.bench_function("vec_into_py", vec_into_py);
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
