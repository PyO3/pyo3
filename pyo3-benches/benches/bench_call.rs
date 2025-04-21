use std::hint::black_box;

use codspeed_criterion_compat::{criterion_group, criterion_main, Bencher, Criterion};

use pyo3::ffi::c_str;
use pyo3::prelude::*;
use pyo3::types::IntoPyDict;

macro_rules! test_module {
    ($py:ident, $code:literal) => {
        PyModule::from_code($py, c_str!($code), c_str!(file!()), c_str!("test_module"))
            .expect("module creation failed")
    };
}

fn bench_call_0(b: &mut Bencher<'_>) {
    Python::with_gil(|py| {
        let module = test_module!(py, "def foo(): pass");

        let foo_module = &module.getattr("foo").unwrap();

        b.iter(|| {
            for _ in 0..1000 {
                black_box(foo_module).call0().unwrap();
            }
        });
    })
}

fn bench_call_1(b: &mut Bencher<'_>) {
    Python::with_gil(|py| {
        let module = test_module!(py, "def foo(a, b, c): pass");

        let foo_module = &module.getattr("foo").unwrap();
        let args = (
            1.into_pyobject(py).unwrap(),
            "s".into_pyobject(py).unwrap(),
            1.23.into_pyobject(py).unwrap(),
        );

        b.iter(|| {
            for _ in 0..1000 {
                black_box(foo_module).call1(args.clone()).unwrap();
            }
        });
    })
}

fn bench_call(b: &mut Bencher<'_>) {
    Python::with_gil(|py| {
        let module = test_module!(py, "def foo(a, b, c, d, e): pass");

        let foo_module = &module.getattr("foo").unwrap();
        let args = (
            1.into_pyobject(py).unwrap(),
            "s".into_pyobject(py).unwrap(),
            1.23.into_pyobject(py).unwrap(),
        );
        let kwargs = [("d", 1), ("e", 42)].into_py_dict(py).unwrap();

        b.iter(|| {
            for _ in 0..1000 {
                black_box(foo_module)
                    .call(args.clone(), Some(&kwargs))
                    .unwrap();
            }
        });
    })
}

fn bench_call_one_arg(b: &mut Bencher<'_>) {
    Python::with_gil(|py| {
        let module = test_module!(py, "def foo(a): pass");

        let foo_module = &module.getattr("foo").unwrap();
        let arg = 1i32.into_pyobject(py).unwrap();

        b.iter(|| {
            for _ in 0..1000 {
                black_box(foo_module).call1((arg.clone(),)).unwrap();
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

        let foo_module = &module.getattr("Foo").unwrap().call0().unwrap();

        b.iter(|| {
            for _ in 0..1000 {
                black_box(foo_module).call_method0("foo").unwrap();
            }
        });
    })
}

fn bench_call_method_1(b: &mut Bencher<'_>) {
    Python::with_gil(|py| {
        let module = test_module!(
            py,
            "
class Foo:
    def foo(self, a, b, c):
        pass
"
        );

        let foo_module = &module.getattr("Foo").unwrap().call0().unwrap();
        let args = (
            1.into_pyobject(py).unwrap(),
            "s".into_pyobject(py).unwrap(),
            1.23.into_pyobject(py).unwrap(),
        );

        b.iter(|| {
            for _ in 0..1000 {
                black_box(foo_module)
                    .call_method1("foo", args.clone())
                    .unwrap();
            }
        });
    })
}

fn bench_call_method(b: &mut Bencher<'_>) {
    Python::with_gil(|py| {
        let module = test_module!(
            py,
            "
class Foo:
    def foo(self, a, b, c, d, e):
        pass
"
        );

        let foo_module = &module.getattr("Foo").unwrap().call0().unwrap();
        let args = (
            1.into_pyobject(py).unwrap(),
            "s".into_pyobject(py).unwrap(),
            1.23.into_pyobject(py).unwrap(),
        );
        let kwargs = [("d", 1), ("e", 42)].into_py_dict(py).unwrap();

        b.iter(|| {
            for _ in 0..1000 {
                black_box(foo_module)
                    .call_method("foo", args.clone(), Some(&kwargs))
                    .unwrap();
            }
        });
    })
}

fn bench_call_method_one_arg(b: &mut Bencher<'_>) {
    Python::with_gil(|py| {
        let module = test_module!(
            py,
            "
class Foo:
    def foo(self, a):
        pass
"
        );

        let foo_module = &module.getattr("Foo").unwrap().call0().unwrap();
        let arg = 1i32.into_pyobject(py).unwrap();

        b.iter(|| {
            for _ in 0..1000 {
                black_box(foo_module)
                    .call_method1("foo", (arg.clone(),))
                    .unwrap();
            }
        });
    })
}

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("call_0", bench_call_0);
    c.bench_function("call_1", bench_call_1);
    c.bench_function("call", bench_call);
    c.bench_function("call_one_arg", bench_call_one_arg);
    c.bench_function("call_method_0", bench_call_method_0);
    c.bench_function("call_method_1", bench_call_method_1);
    c.bench_function("call_method", bench_call_method);
    c.bench_function("call_method_one_arg", bench_call_method_one_arg);
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
