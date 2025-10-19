use std::hint::black_box;

use codspeed_criterion_compat::{criterion_group, criterion_main, Bencher, Criterion};

use pyo3::{
    prelude::*,
    types::{PyDict, PyFloat, PyInt, PyString},
};

fn extract_str_extract_success(bench: &mut Bencher<'_>) {
    Python::attach(|py| {
        let s = PyString::new(py, "Hello, World!").into_any();

        bench.iter(|| black_box(&s).extract::<&str>().unwrap());
    });
}

fn extract_str_extract_fail(bench: &mut Bencher<'_>) {
    Python::attach(|py| {
        let d = PyDict::new(py).into_any();

        bench.iter(|| match black_box(&d).extract::<&str>() {
            Ok(v) => panic!("should err {}", v),
            Err(e) => e,
        });
    });
}

#[cfg(Py_3_10)]
fn extract_str_cast_success(bench: &mut Bencher<'_>) {
    Python::attach(|py| {
        let s = PyString::new(py, "Hello, World!").into_any();

        bench.iter(|| {
            let py_str = black_box(&s).cast::<PyString>().unwrap();
            py_str.to_str().unwrap()
        });
    });
}

fn extract_str_cast_fail(bench: &mut Bencher<'_>) {
    Python::attach(|py| {
        let d = PyDict::new(py).into_any();

        bench.iter(|| match black_box(&d).cast::<PyString>() {
            Ok(v) => panic!("should err {}", v),
            Err(e) => e,
        });
    });
}

fn extract_int_extract_success(bench: &mut Bencher<'_>) {
    Python::attach(|py| {
        let int = 123i32.into_pyobject(py).unwrap();

        bench.iter(|| black_box(&int).extract::<i64>().unwrap());
    });
}

fn extract_int_extract_fail(bench: &mut Bencher<'_>) {
    Python::attach(|py| {
        let d = PyDict::new(py).into_any();

        bench.iter(|| match black_box(&d).extract::<i64>() {
            Ok(v) => panic!("should err {}", v),
            Err(e) => e,
        });
    });
}

fn extract_int_cast_success(bench: &mut Bencher<'_>) {
    Python::attach(|py| {
        let int = 123i32.into_pyobject(py).unwrap();

        bench.iter(|| {
            let py_int = black_box(&int).cast::<PyInt>().unwrap();
            py_int.extract::<i64>().unwrap()
        });
    });
}

fn extract_int_cast_fail(bench: &mut Bencher<'_>) {
    Python::attach(|py| {
        let d = PyDict::new(py).into_any();

        bench.iter(|| match black_box(&d).cast::<PyInt>() {
            Ok(v) => panic!("should err {}", v),
            Err(e) => black_box(e),
        });
    });
}

fn extract_float_extract_success(bench: &mut Bencher<'_>) {
    Python::attach(|py| {
        let float = 23.42f64.into_pyobject(py).unwrap();

        bench.iter(|| black_box(&float).extract::<f64>().unwrap());
    });
}

fn extract_float_extract_fail(bench: &mut Bencher<'_>) {
    Python::attach(|py| {
        let d = PyDict::new(py).into_any();

        bench.iter(|| match black_box(&d).extract::<f64>() {
            Ok(v) => panic!("should err {}", v),
            Err(e) => e,
        });
    });
}

fn extract_float_cast_success(bench: &mut Bencher<'_>) {
    Python::attach(|py| {
        let float = 23.42f64.into_pyobject(py).unwrap();

        bench.iter(|| {
            let py_float = black_box(&float).cast::<PyFloat>().unwrap();
            py_float.value()
        });
    });
}

fn extract_float_cast_fail(bench: &mut Bencher<'_>) {
    Python::attach(|py| {
        let d = PyDict::new(py).into_any();

        bench.iter(|| match black_box(&d).cast::<PyFloat>() {
            Ok(v) => panic!("should err {}", v),
            Err(e) => e,
        });
    });
}

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("extract_str_extract_success", extract_str_extract_success);
    c.bench_function("extract_str_extract_fail", extract_str_extract_fail);
    #[cfg(any(Py_3_10, not(Py_LIMITED_API)))]
    c.bench_function("extract_str_cast_success", extract_str_cast_success);
    c.bench_function("extract_str_cast_fail", extract_str_cast_fail);
    c.bench_function("extract_int_extract_success", extract_int_extract_success);
    c.bench_function("extract_int_extract_fail", extract_int_extract_fail);
    c.bench_function("extract_int_cast_success", extract_int_cast_success);
    c.bench_function("extract_int_cast_fail", extract_int_cast_fail);
    c.bench_function(
        "extract_float_extract_success",
        extract_float_extract_success,
    );
    c.bench_function("extract_float_extract_fail", extract_float_extract_fail);
    c.bench_function(
        "extract_float_cast_success",
        extract_float_cast_success,
    );
    c.bench_function("extract_float_cast_fail", extract_float_cast_fail);
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
