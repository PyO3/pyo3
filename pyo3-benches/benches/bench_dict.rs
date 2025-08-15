use std::collections::{BTreeMap, HashMap};
use std::hint::black_box;

use codspeed_criterion_compat::{criterion_group, criterion_main, Bencher, Criterion};

use pyo3::types::IntoPyDict;
use pyo3::{prelude::*, types::PyMapping};

fn iter_dict(b: &mut Bencher<'_>) {
    Python::attach(|py| {
        const LEN: usize = 100_000;
        let dict = (0..LEN as u64)
            .map(|i| (i, i * 2))
            .into_py_dict(py)
            .unwrap();
        let mut sum = 0;
        b.iter(|| {
            for (k, _v) in &dict {
                let i: u64 = k.extract().unwrap();
                sum += i;
            }
        });
    })
}

fn dict_new(b: &mut Bencher<'_>) {
    Python::attach(|py| {
        const LEN: usize = 50_000;
        b.iter_with_large_drop(|| {
            (0..LEN as u64)
                .map(|i| (i, i * 2))
                .into_py_dict(py)
                .unwrap()
        });
    });
}

fn dict_get_item(b: &mut Bencher<'_>) {
    Python::attach(|py| {
        const LEN: usize = 50_000;
        let dict = (0..LEN as u64)
            .map(|i| (i, i * 2))
            .into_py_dict(py)
            .unwrap();
        let mut sum = 0;
        b.iter(|| {
            for i in 0..LEN {
                sum += dict
                    .get_item(i)
                    .unwrap()
                    .unwrap()
                    .extract::<usize>()
                    .unwrap();
            }
        });
    });
}

fn extract_hashmap(b: &mut Bencher<'_>) {
    Python::attach(|py| {
        const LEN: usize = 100_000;
        let dict = (0..LEN as u64)
            .map(|i| (i, i * 2))
            .into_py_dict(py)
            .unwrap();
        b.iter(|| HashMap::<u64, u64>::extract_bound(&dict));
    });
}

fn extract_btreemap(b: &mut Bencher<'_>) {
    Python::attach(|py| {
        const LEN: usize = 100_000;
        let dict = (0..LEN as u64)
            .map(|i| (i, i * 2))
            .into_py_dict(py)
            .unwrap();
        b.iter(|| BTreeMap::<u64, u64>::extract_bound(&dict));
    });
}

fn extract_hashbrown_map(b: &mut Bencher<'_>) {
    Python::attach(|py| {
        const LEN: usize = 100_000;
        let dict = (0..LEN as u64)
            .map(|i| (i, i * 2))
            .into_py_dict(py)
            .unwrap();
        b.iter(|| hashbrown::HashMap::<u64, u64>::extract_bound(&dict));
    });
}

fn mapping_from_dict(b: &mut Bencher<'_>) {
    Python::attach(|py| {
        const LEN: usize = 100_000;
        let dict = &(0..LEN as u64)
            .map(|i| (i, i * 2))
            .into_py_dict(py)
            .unwrap();
        b.iter(|| black_box(dict).cast::<PyMapping>().unwrap());
    });
}

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("iter_dict", iter_dict);
    c.bench_function("dict_new", dict_new);
    c.bench_function("dict_get_item", dict_get_item);
    c.bench_function("extract_hashmap", extract_hashmap);
    c.bench_function("extract_btreemap", extract_btreemap);
    c.bench_function("mapping_from_dict", mapping_from_dict);
    c.bench_function("extract_hashbrown_map", extract_hashbrown_map);
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
