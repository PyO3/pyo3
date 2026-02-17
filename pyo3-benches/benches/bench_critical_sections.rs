use codspeed_criterion_compat::{criterion_group, criterion_main, Bencher, Criterion};

use pyo3::prelude::*;
use pyo3::sync::critical_section::{with_critical_section, with_critical_section2};
use pyo3::types::PyList;

fn create_cs(b: &mut Bencher<'_>) {
    Python::attach(|py| {
        let lis = PyList::new(py, 0..3).unwrap();
        b.iter(|| {
            with_critical_section(&lis, || {});
        })
    });
}

fn create_cs2(b: &mut Bencher<'_>) {
    Python::attach(|py| {
        let lis1 = PyList::new(py, 0..3).unwrap();
        let lis2 = PyList::new(py, 4..6).unwrap();
        b.iter(|| {
            with_critical_section2(&lis1, &lis2, || {});
        })
    });
}

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("critical_section_creation", create_cs);
    c.bench_function("critical_section_creation2", create_cs2);
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
