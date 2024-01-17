use codspeed_criterion_compat::{criterion_group, criterion_main, Bencher, Criterion};

use pyo3::prelude::*;
use pyo3::types::PySet;
use std::collections::{BTreeSet, HashSet};

fn set_new(b: &mut Bencher<'_>) {
    Python::with_gil(|py| {
        const LEN: usize = 100_000;
        // Create Python objects up-front, so that the benchmark doesn't need to include
        // the cost of allocating LEN Python integers
        let elements: Vec<PyObject> = (0..LEN).map(|i| i.into_py(py)).collect();
        b.iter_with_large_drop(|| PySet::new_bound(py, &elements).unwrap());
    });
}

fn iter_set(b: &mut Bencher<'_>) {
    Python::with_gil(|py| {
        const LEN: usize = 100_000;
        let set = PySet::new_bound(py, &(0..LEN).collect::<Vec<_>>()).unwrap();
        let mut sum = 0;
        b.iter(|| {
            for x in set.iter() {
                let i: u64 = x.extract().unwrap();
                sum += i;
            }
        });
    });
}

fn extract_hashset(b: &mut Bencher<'_>) {
    Python::with_gil(|py| {
        const LEN: usize = 100_000;
        let set = PySet::new_bound(py, &(0..LEN).collect::<Vec<_>>()).unwrap();
        b.iter_with_large_drop(|| HashSet::<u64>::extract(set.as_gil_ref()));
    });
}

fn extract_btreeset(b: &mut Bencher<'_>) {
    Python::with_gil(|py| {
        const LEN: usize = 100_000;
        let set = PySet::new_bound(py, &(0..LEN).collect::<Vec<_>>()).unwrap();
        b.iter_with_large_drop(|| BTreeSet::<u64>::extract(set.as_gil_ref()));
    });
}

#[cfg(feature = "hashbrown")]
fn extract_hashbrown_set(b: &mut Bencher<'_>) {
    Python::with_gil(|py| {
        const LEN: usize = 100_000;
        let set = PySet::new_bound(py, &(0..LEN).collect::<Vec<_>>()).unwrap();
        b.iter_with_large_drop(|| hashbrown::HashSet::<u64>::extract(set.as_gil_ref()));
    });
}

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("set_new", set_new);
    c.bench_function("iter_set", iter_set);
    c.bench_function("extract_hashset", extract_hashset);
    c.bench_function("extract_btreeset", extract_btreeset);

    #[cfg(feature = "hashbrown")]
    c.bench_function("extract_hashbrown_set", extract_hashbrown_set);
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
