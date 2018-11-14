#![feature(test)]
extern crate pyo3;
extern crate test;
use test::Bencher;

use pyo3::{prelude::*, types::IntoPyDict};

#[bench]
fn iter_dict(b: &mut Bencher) {
    let gil = Python::acquire_gil();
    let py = gil.python();
    const LEN: usize = 1_000_000;
    let dict = (0..LEN as u64).map(|i| (i, i * 2)).into_py_dict(py);
    let mut sum = 0;
    b.iter(|| {
        for (k, _v) in dict.iter() {
            let i: u64 = k.extract().unwrap();
            sum += i;
        }
    });
}
