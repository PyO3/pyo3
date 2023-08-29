use codspeed_criterion_compat::{black_box, criterion_group, criterion_main, Bencher, Criterion};

use pyo3::{
    types::{PyDict, PyFloat, PyInt, PyString},
    IntoPy, PyAny, PyObject, Python,
};

fn extract_str_extract_success(bench: &mut Bencher<'_>) {
    Python::with_gil(|py| {
        let s = PyString::new(py, "Hello, World!") as &PyAny;

        bench.iter(|| {
            let v = black_box(s).extract::<&str>().unwrap();
            black_box(v);
        });
    });
}

fn extract_str_extract_fail(bench: &mut Bencher<'_>) {
    Python::with_gil(|py| {
        let d = PyDict::new(py) as &PyAny;

        bench.iter(|| match black_box(d).extract::<&str>() {
            Ok(v) => panic!("should err {}", v),
            Err(e) => black_box(e),
        });
    });
}

fn extract_str_downcast_success(bench: &mut Bencher<'_>) {
    Python::with_gil(|py| {
        let s = PyString::new(py, "Hello, World!") as &PyAny;

        bench.iter(|| {
            let py_str = black_box(s).downcast::<PyString>().unwrap();
            let v = py_str.to_str().unwrap();
            black_box(v);
        });
    });
}

fn extract_str_downcast_fail(bench: &mut Bencher<'_>) {
    Python::with_gil(|py| {
        let d = PyDict::new(py) as &PyAny;

        bench.iter(|| match black_box(d).downcast::<PyString>() {
            Ok(v) => panic!("should err {}", v),
            Err(e) => black_box(e),
        });
    });
}

fn extract_int_extract_success(bench: &mut Bencher<'_>) {
    Python::with_gil(|py| {
        let int_obj: PyObject = 123.into_py(py);
        let int = int_obj.as_ref(py);

        bench.iter(|| {
            let v = black_box(int).extract::<i64>().unwrap();
            black_box(v);
        });
    });
}

fn extract_int_extract_fail(bench: &mut Bencher<'_>) {
    Python::with_gil(|py| {
        let d = PyDict::new(py) as &PyAny;

        bench.iter(|| match black_box(d).extract::<i64>() {
            Ok(v) => panic!("should err {}", v),
            Err(e) => black_box(e),
        });
    });
}

fn extract_int_downcast_success(bench: &mut Bencher<'_>) {
    Python::with_gil(|py| {
        let int_obj: PyObject = 123.into_py(py);
        let int = int_obj.as_ref(py);

        bench.iter(|| {
            let py_int = black_box(int).downcast::<PyInt>().unwrap();
            let v = py_int.extract::<i64>().unwrap();
            black_box(v);
        });
    });
}

fn extract_int_downcast_fail(bench: &mut Bencher<'_>) {
    Python::with_gil(|py| {
        let d = PyDict::new(py) as &PyAny;

        bench.iter(|| match black_box(d).downcast::<PyInt>() {
            Ok(v) => panic!("should err {}", v),
            Err(e) => black_box(e),
        });
    });
}

fn extract_float_extract_success(bench: &mut Bencher<'_>) {
    Python::with_gil(|py| {
        let float_obj: PyObject = 23.42.into_py(py);
        let float = float_obj.as_ref(py);

        bench.iter(|| {
            let v = black_box(float).extract::<f64>().unwrap();
            black_box(v);
        });
    });
}

fn extract_float_extract_fail(bench: &mut Bencher<'_>) {
    Python::with_gil(|py| {
        let d = PyDict::new(py) as &PyAny;

        bench.iter(|| match black_box(d).extract::<f64>() {
            Ok(v) => panic!("should err {}", v),
            Err(e) => black_box(e),
        });
    });
}

fn extract_float_downcast_success(bench: &mut Bencher<'_>) {
    Python::with_gil(|py| {
        let float_obj: PyObject = 23.42.into_py(py);
        let float = float_obj.as_ref(py);

        bench.iter(|| {
            let py_int = black_box(float).downcast::<PyFloat>().unwrap();
            let v = py_int.extract::<f64>().unwrap();
            black_box(v);
        });
    });
}

fn extract_float_downcast_fail(bench: &mut Bencher<'_>) {
    Python::with_gil(|py| {
        let d = PyDict::new(py) as &PyAny;

        bench.iter(|| match black_box(d).downcast::<PyFloat>() {
            Ok(v) => panic!("should err {}", v),
            Err(e) => black_box(e),
        });
    });
}

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("extract_str_extract_success", extract_str_extract_success);
    c.bench_function("extract_str_extract_fail", extract_str_extract_fail);
    c.bench_function("extract_str_downcast_success", extract_str_downcast_success);
    c.bench_function("extract_str_downcast_fail", extract_str_downcast_fail);
    c.bench_function("extract_int_extract_success", extract_int_extract_success);
    c.bench_function("extract_int_extract_fail", extract_int_extract_fail);
    c.bench_function("extract_int_downcast_success", extract_int_downcast_success);
    c.bench_function("extract_int_downcast_fail", extract_int_downcast_fail);
    c.bench_function(
        "extract_float_extract_success",
        extract_float_extract_success,
    );
    c.bench_function("extract_float_extract_fail", extract_float_extract_fail);
    c.bench_function(
        "extract_float_downcast_success",
        extract_float_downcast_success,
    );
    c.bench_function("extract_float_downcast_fail", extract_float_downcast_fail);
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
