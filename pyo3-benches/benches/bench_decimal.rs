use codspeed_criterion_compat::{black_box, criterion_group, criterion_main, Bencher, Criterion};

use pyo3::prelude::*;
use pyo3::types::PyDict;
use rust_decimal::Decimal;

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

        b.iter(|| {
            let _: Decimal = black_box(&py_dec).extract().unwrap();
        });
    })
}

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("decimal_via_extract", decimal_via_extract);
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
