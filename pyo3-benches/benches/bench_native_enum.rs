use std::hint::black_box;

use codspeed_criterion_compat::{criterion_group, criterion_main, Bencher, Criterion};

use pyo3::native_enum::NativeEnum;
use pyo3::prelude::*;
use pyo3::py_native_enum;

/// A simple enum using the default `enum.Enum` base.
#[py_native_enum]
enum Color {
    Red,
    Green,
    Blue,
}

/// An integer enum using `enum.IntEnum`.
#[py_native_enum(base = "IntEnum")]
enum Status {
    Active,
    Inactive,
    Pending,
}

// Measures the PyOnceLock cache-hit path: get + clone_ref + into_bound.
fn bench_py_enum_class(b: &mut Bencher<'_>) {
    Python::attach(|py| {
        b.iter(|| black_box(Color::py_enum_class(py).unwrap()));
    });
}

fn bench_int_enum_class(b: &mut Bencher<'_>) {
    Python::attach(|py| {
        b.iter(|| black_box(Status::py_enum_class(py).unwrap()));
    });
}

fn bench_to_py_member(b: &mut Bencher<'_>) {
    Python::attach(|py| {
        b.iter_with_large_drop(|| Color::Green.to_py_member(py).unwrap());
    });
}

fn bench_int_enum_to_py_member(b: &mut Bencher<'_>) {
    Python::attach(|py| {
        b.iter_with_large_drop(|| Status::Active.to_py_member(py).unwrap());
    });
}

fn bench_from_py_member(b: &mut Bencher<'_>) {
    Python::attach(|py| {
        let obj = Color::Blue.to_py_member(py).unwrap();
        b.iter(|| Color::from_py_member(black_box(&obj)).unwrap());
    });
}

fn bench_int_enum_from_py_member(b: &mut Bencher<'_>) {
    Python::attach(|py| {
        let obj = Status::Pending.to_py_member(py).unwrap();
        b.iter(|| Status::from_py_member(black_box(&obj)).unwrap());
    });
}

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("native_enum_py_enum_class", bench_py_enum_class);
    c.bench_function("native_enum_int_enum_class", bench_int_enum_class);
    c.bench_function("native_enum_to_py_member", bench_to_py_member);
    c.bench_function("native_enum_int_enum_to_py_member", bench_int_enum_to_py_member);
    c.bench_function("native_enum_from_py_member", bench_from_py_member);
    c.bench_function("native_enum_int_enum_from_py_member", bench_int_enum_from_py_member);
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
