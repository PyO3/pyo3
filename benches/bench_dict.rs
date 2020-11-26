#![feature(test)]

extern crate test;
use pyo3::prelude::*;
use pyo3::types::IntoPyDict;
use std::collections::{BTreeMap, HashMap};
use test::Bencher;

#[bench]
fn iter_dict(b: &mut Bencher) {
    let gil = Python::acquire_gil();
    let py = gil.python();
    const LEN: usize = 100_000;
    let dict = (0..LEN as u64).map(|i| (i, i * 2)).into_py_dict(py);
    let mut sum = 0;
    b.iter(|| {
        for (k, _v) in dict.iter() {
            let i: u64 = k.extract().unwrap();
            sum += i;
        }
    });
}

#[bench]
fn dict_get_item(b: &mut Bencher) {
    let gil = Python::acquire_gil();
    let py = gil.python();
    const LEN: usize = 50_000;
    let dict = (0..LEN as u64).map(|i| (i, i * 2)).into_py_dict(py);
    let mut sum = 0;
    b.iter(|| {
        for i in 0..LEN {
            sum += dict.get_item(i).unwrap().extract::<usize>().unwrap();
        }
    });
}

#[bench]
fn extract_hashmap(b: &mut Bencher) {
    let gil = Python::acquire_gil();
    let py = gil.python();
    const LEN: usize = 100_000;
    let dict = (0..LEN as u64).map(|i| (i, i * 2)).into_py_dict(py);
    b.iter(|| HashMap::<u64, u64>::extract(dict));
}

#[bench]
fn extract_btreemap(b: &mut Bencher) {
    let gil = Python::acquire_gil();
    let py = gil.python();
    const LEN: usize = 100_000;
    let dict = (0..LEN as u64).map(|i| (i, i * 2)).into_py_dict(py);
    b.iter(|| BTreeMap::<u64, u64>::extract(dict));
}

#[bench]
#[cfg(feature = "hashbrown")]
fn extract_hashbrown_map(b: &mut Bencher) {
    let gil = Python::acquire_gil();
    let py = gil.python();
    const LEN: usize = 100_000;
    let dict = (0..LEN as u64).map(|i| (i, i * 2)).into_py_dict(py);
    b.iter(|| hashbrown::HashMap::<u64, u64>::extract(dict));
}
