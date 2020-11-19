#![feature(test)]

extern crate test;
use pyo3::prelude::*;
use test::Bencher;

macro_rules! test_module {
    ($py:ident, $code:literal) => {
        PyModule::from_code($py, indoc::indoc!($code), file!(), "test_module")
            .expect("module creation failed")
    };
}

#[bench]
fn bench_call_0(b: &mut Bencher) {
    Python::with_gil(|py| {
        let module = test_module!(
            py,
            r#"
            def foo(): pass
        "#
        );

        let foo = module.getattr("foo").unwrap();

        b.iter(|| {
            for _ in 0..1000 {
                foo.call0().unwrap();
            }
        });
    })
}

#[bench]
fn bench_call_method_0(b: &mut Bencher) {
    Python::with_gil(|py| {
        let module = test_module!(
            py,
            r#"
            class Foo:
                def foo(self): pass
        "#
        );

        let foo = module.getattr("Foo").unwrap().call0().unwrap();

        b.iter(|| {
            for _ in 0..1000 {
                foo.call_method0("foo").unwrap();
            }
        });
    })
}
