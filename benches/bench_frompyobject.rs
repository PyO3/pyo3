use criterion::{criterion_group, criterion_main, Bencher, Criterion};

use pyo3::{prelude::*, types::PyString};

#[derive(FromPyObject)]
enum ManyTypes {
    Int(i32),
    Bytes(Vec<u8>),
    String(String),
}

fn enum_from_pyobject(b: &mut Bencher) {
    Python::with_gil(|py| {
        let obj = PyString::new(py, "hello world");
        b.iter(|| {
            let _: ManyTypes = obj.extract().unwrap();
        });
    })
}

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("enum_from_pyobject", enum_from_pyobject);
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
