use codspeed_criterion_compat::{criterion_group, criterion_main, Bencher, Criterion};
use pyo3::{py_format, Python};
use std::hint::black_box;

fn format_complex(b: &mut Bencher<'_>) {
    Python::attach(|py| {
        b.iter(|| {
            let value = (black_box(42), black_box("foo"), [0; 0]);
            py_format!(py, "This is some complex value: {value:?}").unwrap()
        });
    });
}

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("format_complex", format_complex);
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
