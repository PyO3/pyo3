use codspeed_criterion_compat::{black_box, criterion_group, criterion_main, Bencher, Criterion};

use pyo3::{
    prelude::*,
    types::{PyFloat, PyList, PyString},
};

#[derive(FromPyObject)]
enum ManyTypes {
    Int(i32),
    Bytes(Vec<u8>),
    String(String),
}

fn enum_from_pyobject(b: &mut Bencher<'_>) {
    Python::with_gil(|py| {
        let obj = PyString::new(py, "hello world");
        b.iter(|| {
            let _: ManyTypes = obj.extract().unwrap();
        });
    })
}

fn list_via_downcast(b: &mut Bencher<'_>) {
    Python::with_gil(|py| {
        let any: &PyAny = PyList::empty(py).into();

        b.iter(|| {
            let _list: &PyList = black_box(any).downcast().unwrap();
        });
    })
}

fn list_via_extract(b: &mut Bencher<'_>) {
    Python::with_gil(|py| {
        let any: &PyAny = PyList::empty(py).into();

        b.iter(|| {
            let _list: &PyList = black_box(any).extract().unwrap();
        });
    })
}

fn not_a_list_via_downcast(b: &mut Bencher<'_>) {
    Python::with_gil(|py| {
        let any: &PyAny = PyString::new(py, "foobar").into();

        b.iter(|| {
            black_box(any).downcast::<PyList>().unwrap_err();
        });
    })
}

fn not_a_list_via_extract(b: &mut Bencher<'_>) {
    Python::with_gil(|py| {
        let any: &PyAny = PyString::new(py, "foobar").into();

        b.iter(|| {
            black_box(any).extract::<&PyList>().unwrap_err();
        });
    })
}

#[derive(FromPyObject)]
enum ListOrNotList<'a> {
    List(&'a PyList),
    NotList(&'a PyAny),
}

fn not_a_list_via_extract_enum(b: &mut Bencher<'_>) {
    Python::with_gil(|py| {
        let any: &PyAny = PyString::new(py, "foobar").into();

        b.iter(|| match black_box(any).extract::<ListOrNotList<'_>>() {
            Ok(ListOrNotList::List(_list)) => panic!(),
            Ok(ListOrNotList::NotList(_any)) => (),
            Err(_) => panic!(),
        });
    })
}

fn f64_from_pyobject(b: &mut Bencher<'_>) {
    Python::with_gil(|py| {
        let obj = PyFloat::new(py, 1.234);
        b.iter(|| {
            let _: f64 = obj.extract().unwrap();
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
    c.bench_function("f64_from_pyobject", f64_from_pyobject);
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
