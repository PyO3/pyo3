use codspeed_criterion_compat::{black_box, criterion_group, criterion_main, Bencher, Criterion};

use pyo3::prelude::*;
use pyo3::types::PyDict;

use num_bigint::BigInt;

fn extract_bigint_extract_fail(bench: &mut Bencher<'_>) {
    Python::with_gil(|py| {
        let d = PyDict::new_bound(py).into_any();

        bench.iter(|| match black_box(&d).extract::<BigInt>() {
            Ok(v) => panic!("should err {}", v),
            Err(e) => black_box(e),
        });
    });
}

fn extract_bigint_small(bench: &mut Bencher<'_>) {
    Python::with_gil(|py| {
        let int = py.eval_bound("-42", None, None).unwrap();

        bench.iter(|| {
            let v = black_box(&int).extract::<BigInt>().unwrap();
            black_box(v);
        });
    });
}

fn extract_bigint_big_negative(bench: &mut Bencher<'_>) {
    Python::with_gil(|py| {
        let int = py.eval_bound("-10**300", None, None).unwrap();

        bench.iter(|| {
            let v = black_box(&int).extract::<BigInt>().unwrap();
            black_box(v);
        });
    });
}

fn extract_bigint_big_positive(bench: &mut Bencher<'_>) {
    Python::with_gil(|py| {
        let int = py.eval_bound("10**300", None, None).unwrap();

        bench.iter(|| {
            let v = black_box(&int).extract::<BigInt>().unwrap();
            black_box(v);
        });
    });
}

fn extract_bigint_huge_negative(bench: &mut Bencher<'_>) {
    Python::with_gil(|py| {
        let int = py.eval_bound("-10**3000", None, None).unwrap();

        bench.iter(|| {
            let v = black_box(&int).extract::<BigInt>().unwrap();
            black_box(v);
        });
    });
}

fn extract_bigint_huge_positive(bench: &mut Bencher<'_>) {
    Python::with_gil(|py| {
        let int = py.eval_bound("10**3000", None, None).unwrap();

        bench.iter(|| {
            let v = black_box(&int).extract::<BigInt>().unwrap();
            black_box(v);
        });
    });
}

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("extract_bigint_extract_fail", extract_bigint_extract_fail);
    c.bench_function("extract_bigint_small", extract_bigint_small);
    c.bench_function("extract_bigint_big_negative", extract_bigint_big_negative);
    c.bench_function("extract_bigint_big_positive", extract_bigint_big_positive);
    c.bench_function("extract_bigint_huge_negative", extract_bigint_huge_negative);
    c.bench_function("extract_bigint_huge_positive", extract_bigint_huge_positive);
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
