#![feature(test)]

extern crate test;
use pyo3::experimental::objects::{IntoPyDict, PyList, PyTuple, PySet};
use pyo3::experimental::prelude::*;
use std::collections::{BTreeMap, HashMap, BTreeSet, HashSet};
use test::Bencher;

macro_rules! test_module {
    ($py:ident, $code:literal) => {
        PyModule::from_code($py, indoc::indoc!($code), file!(), "test_module")
            .expect("module creation failed")
    };
}

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
    let dict_ref = &*dict;
    let mut sum = 0;
    b.iter(|| {
        for i in 0..LEN {
            sum += dict_ref.get_item(i).unwrap().extract::<u64>().unwrap();
        }
    });
}

#[bench]
fn extract_hashmap(b: &mut Bencher) {
    let gil = Python::acquire_gil();
    let py = gil.python();
    const LEN: usize = 100_000;
    let dict = (0..LEN as u64).map(|i| (i, i * 2)).into_py_dict(py);
    b.iter(|| HashMap::<u64, u64>::extract(&dict));
}

#[bench]
fn extract_btreemap(b: &mut Bencher) {
    let gil = Python::acquire_gil();
    let py = gil.python();
    const LEN: usize = 100_000;
    let dict = (0..LEN as u64).map(|i| (i, i * 2)).into_py_dict(py);
    b.iter(|| BTreeMap::<u64, u64>::extract(&dict));
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

#[bench]
fn iter_set(b: &mut Bencher) {
    let gil = Python::acquire_gil();
    let py = gil.python();
    const LEN: usize = 100_000;
    let set = PySet::new(py, &(0..LEN).collect::<Vec<_>>()).unwrap();
    let mut sum = 0;
    b.iter(|| {
        for x in set.iter() {
            let i: u64 = x.extract().unwrap();
            sum += i;
        }
    });
}

#[bench]
fn extract_hashset(b: &mut Bencher) {
    let gil = Python::acquire_gil();
    let py = gil.python();
    const LEN: usize = 100_000;
    let set = PySet::new(py, &(0..LEN).collect::<Vec<_>>()).unwrap();
    b.iter(|| HashSet::<u64>::extract(&set));
}

#[bench]
fn extract_btreeset(b: &mut Bencher) {
    let gil = Python::acquire_gil();
    let py = gil.python();
    const LEN: usize = 100_000;
    let set = PySet::new(py, &(0..LEN).collect::<Vec<_>>()).unwrap();
    b.iter(|| BTreeSet::<u64>::extract(&set));
}

#[bench]
#[cfg(feature = "hashbrown")]
fn extract_hashbrown_set(b: &mut Bencher) {
    let gil = Python::acquire_gil();
    let py = gil.python();
    const LEN: usize = 100_000;
    let set = PySet::new(py, &(0..LEN).collect::<Vec<_>>()).unwrap();
    b.iter(|| hashbrown::HashSet::<u64>::extract(set));
}

#[bench]
fn iter_tuple(b: &mut Bencher) {
    let gil = Python::acquire_gil();
    let py = gil.python();
    const LEN: usize = 100_000;
    let tuple = PyTuple::new(py, 0..LEN);
    let mut sum = 0;
    b.iter(|| {
        for x in tuple.iter() {
            let i: u64 = x.extract().unwrap();
            sum += i;
        }
    });
}

#[bench]
fn tuple_get_item(b: &mut Bencher) {
    let gil = Python::acquire_gil();
    let py = gil.python();
    const LEN: usize = 50_000;
    let tuple = PyTuple::new(py, 0..LEN);
    let mut sum = 0;
    b.iter(|| {
        for i in 0..LEN {
            sum += tuple.get_item(i).extract::<usize>().unwrap();
        }
    });
}
