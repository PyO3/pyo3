use std::hint::black_box;

use codspeed_criterion_compat::{criterion_group, criterion_main, Bencher, Criterion};

use pyo3::{
    prelude::*,
    types::PyInt, conversion::IntoPyObject
};

fn into_u128(b: &mut Bencher<'_>, value: u128) {
    Python::attach(|py| {
        b.iter_with_large_drop(|| black_box(value).into_pyobject(py));
    });
}

fn into_i128(b: &mut Bencher<'_>, value: i128) {
    Python::attach(|py| {
        b.iter_with_large_drop(|| black_box(value).into_pyobject(py));
    });
}

fn extract_u128(b: &mut Bencher<'_>, value: u128) {
    Python::attach(|py| {
        let obj: Bound<'_, PyInt> = value.into_pyobject(py).unwrap();
        b.iter(|| {
            let v: u128 = black_box(&obj).extract().unwrap();
            black_box(v)
        });
    });
}

fn extract_i128(b: &mut Bencher<'_>, value: i128) {
    Python::attach(|py| {
        let obj: Bound<'_, PyInt> = value.into_pyobject(py).unwrap();
        b.iter(|| {
            let v: i128 = black_box(&obj).extract().unwrap();
            black_box(v)
        });
    });
}

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("into_u128_zero", |b| into_u128(b, 0));
    c.bench_function("into_u128_small", |b| into_u128(b, 42));
    c.bench_function("into_u128_u32_max", |b| into_u128(b, u32::MAX as u128));
    c.bench_function("into_u128_u64_max", |b| into_u128(b, u64::MAX as u128));
    c.bench_function("into_u128_max", |b| into_u128(b, u128::MAX));

    c.bench_function("into_i128_zero", |b| into_i128(b, 0));
    c.bench_function("into_i128_small_pos", |b| into_i128(b, 42));
    c.bench_function("into_i128_small_neg", |b| into_i128(b, -42));
    c.bench_function("into_i128_pos_max", |b| into_i128(b, i128::MAX));
    c.bench_function("into_i128_neg_min", |b| into_i128(b, i128::MIN));

    c.bench_function("extract_u128_zero", |b| extract_u128(b, 0));
    c.bench_function("extract_u128_small", |b| extract_u128(b, 42));
    c.bench_function("extract_u128_u32_max", |b| extract_u128(b, u32::MAX as u128));
    c.bench_function("extract_u128_u64_max", |b| extract_u128(b, u64::MAX as u128));
    c.bench_function("extract_u128_max", |b| extract_u128(b, u128::MAX));

    c.bench_function("extract_i128_zero", |b| extract_i128(b, 0));
    c.bench_function("extract_i128_small_pos", |b| extract_i128(b, 42));
    c.bench_function("extract_i128_small_neg", |b| extract_i128(b, -42));
    c.bench_function("extract_i128_pos_max", |b| extract_i128(b, i128::MAX));
    c.bench_function("extract_i128_neg_min", |b| extract_i128(b, i128::MIN));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
