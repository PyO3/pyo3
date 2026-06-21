use std::hint::black_box;

use codspeed_criterion_compat::{criterion_group, criterion_main, Bencher, Criterion};
use num_bigint::{BigInt, BigUint};

use pyo3::conversion::IntoPyObject;
use pyo3::prelude::*;
use pyo3::types::PyDict;

fn extract_bigint_extract_fail(bench: &mut Bencher<'_>) {
    Python::attach(|py| {
        let d = PyDict::new(py).into_any();

        bench.iter(|| match black_box(&d).extract::<BigInt>() {
            Ok(v) => panic!("should err {}", v),
            Err(e) => e,
        });
    });
}

fn extract_bigint(bench: &mut Bencher<'_>, value: &BigInt) {
    Python::attach(|py| {
        let int = value.into_pyobject(py).unwrap();
        bench.iter_with_large_drop(|| black_box(&int).extract::<BigInt>().unwrap());
    });
}

fn extract_biguint(bench: &mut Bencher<'_>, value: &BigUint) {
    Python::attach(|py| {
        let int = value.into_pyobject(py).unwrap();
        bench.iter_with_large_drop(|| black_box(&int).extract::<BigUint>().unwrap());
    });
}

fn extract_biguint_negative_fail(bench: &mut Bencher<'_>) {
    Python::attach(|py| {
        let int = py.eval(c"-10**300", None, None).unwrap();

        bench.iter(|| match black_box(&int).extract::<BigUint>() {
            Ok(v) => panic!("should err {}", v),
            Err(e) => e,
        });
    });
}

fn into_bigint(bench: &mut Bencher<'_>, value: &BigInt) {
    Python::attach(|py| {
        bench.iter_with_large_drop(|| black_box(value).into_pyobject(py).unwrap());
    });
}

fn into_biguint(bench: &mut Bencher<'_>, value: &BigUint) {
    Python::attach(|py| {
        bench.iter_with_large_drop(|| black_box(value).into_pyobject(py).unwrap());
    });
}

fn criterion_benchmark(c: &mut Criterion) {
    let bigint_cases = [
        ("small", BigInt::from(-42)),
        ("big_negative", -(BigInt::from(10u8).pow(300))),
        ("big_positive", BigInt::from(10u8).pow(300)),
        ("huge_negative", -(BigInt::from(10u8).pow(3000))),
        ("huge_positive", BigInt::from(10u8).pow(3000)),
    ];

    let biguint_cases = [
        ("zero", BigUint::from(0u8)),
        ("small", BigUint::from(42u8)),
        ("big", BigUint::from(10u8).pow(300)),
        ("huge", BigUint::from(10u8).pow(3000)),
    ];

    c.bench_function("extract_bigint_extract_fail", extract_bigint_extract_fail);

    for (name, value) in &bigint_cases {
        c.bench_function(&format!("extract_bigint_{name}"), |b| extract_bigint(b, value));
    }

    c.bench_function("extract_biguint_negative_fail", extract_biguint_negative_fail);

    for (name, value) in &biguint_cases {
        c.bench_function(&format!("extract_biguint_{name}"), |b| extract_biguint(b, value));
    }

    for (name, value) in &bigint_cases {
        c.bench_function(&format!("into_bigint_{name}"), |b| into_bigint(b, value));
    }

    for (name, value) in &biguint_cases {
        c.bench_function(&format!("into_biguint_{name}"), |b| into_biguint(b, value));
    }
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
