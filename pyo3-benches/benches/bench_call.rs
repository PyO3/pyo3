use codspeed_criterion_compat::{criterion_group, criterion_main, Bencher, Criterion};

use pyo3::prelude::*;

macro_rules! test_module {
    ($py:ident, $code:literal) => {
        PyModule::from_code($py, $code, file!(), "test_module").expect("module creation failed")
    };
}

fn bench_call_0(b: &mut Bencher<'_>) {
    Python::with_gil(|py| {
        let module = test_module!(py, "def foo(): pass");

        let foo_module = module.getattr("foo").unwrap();

        b.iter(|| {
            for _ in 0..1000 {
                foo_module.call0().unwrap();
            }
        });
    })
}

fn bench_call_method_0(b: &mut Bencher<'_>) {
    Python::with_gil(|py| {
        let module = test_module!(
            py,
            "
class Foo:
    def foo(self):
        pass
"
        );

        let foo_module = module.getattr("Foo").unwrap().call0().unwrap();

        b.iter(|| {
            for _ in 0..1000 {
                foo_module.call_method0("foo").unwrap();
            }
        });
    })
}

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("call_0", bench_call_0);
    c.bench_function("call_method_0", bench_call_method_0);
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
