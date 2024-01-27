use codspeed_criterion_compat::{criterion_group, criterion_main, Bencher, Criterion};

use pyo3::prelude::*;
use pyo3::types::{PyList, PySequence, PyTuple};

fn iter_tuple(b: &mut Bencher<'_>) {
    Python::with_gil(|py| {
        const LEN: usize = 100_000;
        let tuple = PyTuple::new_bound(py, 0..LEN);
        let mut sum = 0;
        b.iter(|| {
            for x in tuple.iter_borrowed() {
                let i: u64 = x.extract().unwrap();
                sum += i;
            }
        });
    });
}

fn tuple_new(b: &mut Bencher<'_>) {
    Python::with_gil(|py| {
        const LEN: usize = 50_000;
        b.iter_with_large_drop(|| PyTuple::new_bound(py, 0..LEN));
    });
}

fn tuple_get_item(b: &mut Bencher<'_>) {
    Python::with_gil(|py| {
        const LEN: usize = 50_000;
        let tuple = PyTuple::new_bound(py, 0..LEN);
        let mut sum = 0;
        b.iter(|| {
            for i in 0..LEN {
                sum += tuple.get_item(i).unwrap().extract::<usize>().unwrap();
            }
        });
    });
}

#[cfg(not(any(Py_LIMITED_API, PyPy)))]
fn tuple_get_item_unchecked(b: &mut Bencher<'_>) {
    Python::with_gil(|py| {
        const LEN: usize = 50_000;
        let tuple = PyTuple::new_bound(py, 0..LEN);
        let mut sum = 0;
        b.iter(|| {
            for i in 0..LEN {
                unsafe {
                    sum += tuple.get_item_unchecked(i).extract::<usize>().unwrap();
                }
            }
        });
    });
}

fn tuple_get_borrowed_item(b: &mut Bencher<'_>) {
    Python::with_gil(|py| {
        const LEN: usize = 50_000;
        let tuple = PyTuple::new_bound(py, 0..LEN);
        let mut sum = 0;
        b.iter(|| {
            for i in 0..LEN {
                sum += tuple
                    .get_borrowed_item(i)
                    .unwrap()
                    .extract::<usize>()
                    .unwrap();
            }
        });
    });
}

#[cfg(not(any(Py_LIMITED_API, PyPy)))]
fn tuple_get_borrowed_item_unchecked(b: &mut Bencher<'_>) {
    Python::with_gil(|py| {
        const LEN: usize = 50_000;
        let tuple = PyTuple::new_bound(py, 0..LEN);
        let mut sum = 0;
        b.iter(|| {
            for i in 0..LEN {
                unsafe {
                    sum += tuple
                        .get_borrowed_item_unchecked(i)
                        .extract::<usize>()
                        .unwrap();
                }
            }
        });
    });
}

fn sequence_from_tuple(b: &mut Bencher<'_>) {
    Python::with_gil(|py| {
        const LEN: usize = 50_000;
        let tuple = PyTuple::new_bound(py, 0..LEN).to_object(py);
        b.iter(|| tuple.extract::<&PySequence>(py).unwrap());
    });
}

fn tuple_new_list(b: &mut Bencher<'_>) {
    Python::with_gil(|py| {
        const LEN: usize = 50_000;
        let tuple = PyTuple::new_bound(py, 0..LEN);
        b.iter_with_large_drop(|| PyList::new_bound(py, tuple.iter_borrowed()));
    });
}

fn tuple_to_list(b: &mut Bencher<'_>) {
    Python::with_gil(|py| {
        const LEN: usize = 50_000;
        let tuple = PyTuple::new_bound(py, 0..LEN);
        b.iter_with_large_drop(|| tuple.to_list());
    });
}

fn tuple_into_py(b: &mut Bencher<'_>) {
    Python::with_gil(|py| {
        b.iter(|| -> PyObject { (1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12).into_py(py) });
    });
}

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("iter_tuple", iter_tuple);
    c.bench_function("tuple_new", tuple_new);
    c.bench_function("tuple_get_item", tuple_get_item);
    #[cfg(not(any(Py_LIMITED_API, PyPy)))]
    c.bench_function("tuple_get_item_unchecked", tuple_get_item_unchecked);
    c.bench_function("tuple_get_borrowed_item", tuple_get_borrowed_item);
    #[cfg(not(any(Py_LIMITED_API, PyPy)))]
    c.bench_function(
        "tuple_get_borrowed_item_unchecked",
        tuple_get_borrowed_item_unchecked,
    );
    c.bench_function("sequence_from_tuple", sequence_from_tuple);
    c.bench_function("tuple_new_list", tuple_new_list);
    c.bench_function("tuple_to_list", tuple_to_list);
    c.bench_function("tuple_into_py", tuple_into_py);
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
