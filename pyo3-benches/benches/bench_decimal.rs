use std::hint::black_box;

use codspeed_criterion_compat::{criterion_group, criterion_main, Bencher, Criterion};
use rust_decimal::Decimal;

use pyo3::prelude::*;
use pyo3::types::PyDict;

fn decimal_via_extract(b: &mut Bencher<'_>) {
    Python::with_gil(|py| {
        let locals = PyDict::new_bound(py);
        py.run_bound(
            r#"
import decimal
py_dec = decimal.Decimal("0.0")
"#,
            None,
            Some(&locals),
        )
        .unwrap();
        let py_dec = locals.get_item("py_dec").unwrap().unwrap();

        b.iter(|| black_box(&py_dec).extract::<Decimal>().unwrap());
    })
}

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("decimal_via_extract", decimal_via_extract);
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
