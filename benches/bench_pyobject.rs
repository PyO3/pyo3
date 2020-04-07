#![feature(test)]

extern crate test;
use pyo3::prelude::*;
use test::Bencher;

#[bench]
fn drop_many_objects(b: &mut Bencher) {
    let gil = Python::acquire_gil();
    let py = gil.python();
    b.iter(|| {
        for _ in 0..1000 {
            std::mem::drop(py.None());
        }
    });
}
