use std::hint::black_box;

use codspeed_criterion_compat::{criterion_group, criterion_main, Bencher, Criterion};

use pyo3::{
    prelude::*,
    types::{PyList, PyString},
};

#[derive(FromPyObject)]
#[allow(dead_code)]
enum ManyTypes {
    Int(i32),
    Bytes(Vec<u8>),
    String(String),
}

fn enum_from_pyobject(b: &mut Bencher<'_>) {
    Python::with_gil(|py| {
        let any = PyString::new_bound(py, "hello world").into_any();

        b.iter(|| black_box(&any).extract::<ManyTypes>().unwrap());
    })
}

fn list_via_downcast(b: &mut Bencher<'_>) {
    Python::with_gil(|py| {
        let any = PyList::empty_bound(py).into_any();

        b.iter(|| black_box(&any).downcast::<PyList>().unwrap());
    })
}

fn list_via_extract(b: &mut Bencher<'_>) {
    Python::with_gil(|py| {
        let any = PyList::empty_bound(py).into_any();

        b.iter(|| black_box(&any).extract::<Bound<'_, PyList>>().unwrap());
    })
}

fn not_a_list_via_downcast(b: &mut Bencher<'_>) {
    Python::with_gil(|py| {
        let any = PyString::new_bound(py, "foobar").into_any();

        b.iter(|| black_box(&any).downcast::<PyList>().unwrap_err());
    })
}

fn not_a_list_via_extract(b: &mut Bencher<'_>) {
    Python::with_gil(|py| {
        let any = PyString::new_bound(py, "foobar").into_any();

        b.iter(|| black_box(&any).extract::<Bound<'_, PyList>>().unwrap_err());
    })
}

#[derive(FromPyObject)]
enum ListOrNotList<'a> {
    List(Bound<'a, PyList>),
    NotList(Bound<'a, PyAny>),
}

fn not_a_list_via_extract_enum(b: &mut Bencher<'_>) {
    Python::with_gil(|py| {
        let any = PyString::new_bound(py, "foobar").into_any();

        b.iter(|| match black_box(&any).extract::<ListOrNotList<'_>>() {
            Ok(ListOrNotList::List(_list)) => panic!(),
            Ok(ListOrNotList::NotList(any)) => any,
            Err(_) => panic!(),
        });
    })
}

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("enum_from_pyobject", enum_from_pyobject);

    c.bench_function("list_via_downcast", list_via_downcast);

    c.bench_function("list_via_extract", list_via_extract);

    c.bench_function("not_a_list_via_downcast", not_a_list_via_downcast);
    c.bench_function("not_a_list_via_extract", not_a_list_via_extract);
    c.bench_function("not_a_list_via_extract_enum", not_a_list_via_extract_enum);
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
