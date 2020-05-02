#![feature(test)]

extern crate test;
use pyo3::prelude::*;
use pyo3::types::PyList;
use test::Bencher;

#[bench]
fn iter_list(b: &mut Bencher) {
    let gil = Python::acquire_gil();
    let py = gil.python();
    const LEN: usize = 100_000;
    let list = PyList::new(py, 0..LEN);
    let mut sum = 0;
    b.iter(|| {
        for x in list.iter() {
            let i: u64 = x.extract().unwrap();
            sum += i;
        }
    });
}

#[bench]
fn list_get_item(b: &mut Bencher) {
    let gil = Python::acquire_gil();
    let py = gil.python();
    const LEN: usize = 50_000;
    let list = PyList::new(py, 0..LEN);
    let mut sum = 0;
    b.iter(|| {
        for i in 0..LEN {
            sum += list.get_item(i as isize).extract::<usize>().unwrap();
        }
    });
}
