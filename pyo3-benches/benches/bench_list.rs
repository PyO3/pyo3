use std::hint::black_box;

use codspeed_criterion_compat::{criterion_group, criterion_main, Bencher, Criterion};

use pyo3::prelude::*;
use pyo3::types::{PyList, PySequence};

fn iter_list(b: &mut Bencher<'_>) {
    Python::with_gil(|py| {
        const LEN: usize = 100_000;
        let list = PyList::new(py, 0..LEN).unwrap();
        let mut sum = 0;
        b.iter(|| {
            for x in &list {
                let i: u64 = x.extract().unwrap();
                sum += i;
            }
        });
    });
}

fn list_new(b: &mut Bencher<'_>) {
    Python::with_gil(|py| {
        const LEN: usize = 50_000;
        b.iter_with_large_drop(|| PyList::new(py, 0..LEN));
    });
}

fn list_get_item(b: &mut Bencher<'_>) {
    Python::with_gil(|py| {
        const LEN: usize = 50_000;
        let list = PyList::new(py, 0..LEN).unwrap();
        let mut sum = 0;
        b.iter(|| {
            for i in 0..LEN {
                sum += list.get_item(i).unwrap().extract::<usize>().unwrap();
            }
        });
    });
}

fn list_nth(b: &mut Bencher<'_>) {
    Python::with_gil(|py| {
        const LEN: usize = 50;
        let list = PyList::new(py, 0..LEN).unwrap();
        let mut sum = 0;
        b.iter(|| {
            for i in 0..LEN {
                sum += list.iter().nth(i).unwrap().extract::<usize>().unwrap();
            }
        });
    });
}

fn list_nth_back(b: &mut Bencher<'_>) {
    Python::with_gil(|py| {
        const LEN: usize = 50;
        let list = PyList::new(py, 0..LEN).unwrap();
        let mut sum = 0;
        b.iter(|| {
            for i in 0..LEN {
                sum += list.iter().nth_back(i).unwrap().extract::<usize>().unwrap();
            }
        });
    });
}

#[cfg(not(Py_LIMITED_API))]
fn list_get_item_unchecked(b: &mut Bencher<'_>) {
    Python::with_gil(|py| {
        const LEN: usize = 50_000;
        let list = PyList::new(py, 0..LEN).unwrap();
        let mut sum = 0;
        b.iter(|| {
            for i in 0..LEN {
                unsafe {
                    sum += list.get_item_unchecked(i).extract::<usize>().unwrap();
                }
            }
        });
    });
}

fn sequence_from_list(b: &mut Bencher<'_>) {
    Python::with_gil(|py| {
        const LEN: usize = 50_000;
        let list = &PyList::new(py, 0..LEN).unwrap();
        b.iter(|| black_box(list).downcast::<PySequence>().unwrap());
    });
}

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("iter_list", iter_list);
    c.bench_function("list_new", list_new);
    c.bench_function("list_nth", list_nth);
    c.bench_function("list_nth_back", list_nth_back);
    c.bench_function("list_get_item", list_get_item);
    #[cfg(not(any(Py_LIMITED_API, Py_GIL_DISABLED)))]
    c.bench_function("list_get_item_unchecked", list_get_item_unchecked);
    c.bench_function("sequence_from_list", sequence_from_list);
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
