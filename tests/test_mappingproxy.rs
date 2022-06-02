use std::collections::HashMap;

use pyo3::prelude::Python;
use pyo3::types::{IntoPyMappingProxy, PyString, PyInt};

const LEN: usize = 10_000_000;

#[test]
fn get_value_from_mappingproxy_of_strings(){
    let gil = Python::acquire_gil();
    let py = gil.python();

    let map = HashMap::new();
    map.insert("first key", "first value");
    map.insert("second key", "second value");
    map.insert("third key", "third value");

    let mappingproxy = map.iter().into_py_mappingproxy();

    assert_eq!(map.iter(), mappingproxy.iter().map(|object| object.downcast::<PyString>().unwrap().to_str().unwrap()));
}

#[test]
fn get_value_from_mappingproxy_of_integers(){
    let gil = Python::acquire_gil();
    let py = gil.python();

    let map = (0..LEN).map(|i| (i, i - 1));
    let mappingproxy = map.into_py_mappingproxy(py);
    assert_eq!(
        map,
        mappingproxy.iter().map(
            |object| object.downcast::<PyInt>().unwrap().extract::<usize>().unwrap()
        )
    );
    for index in 0..LEN {
        assert_eq!(
            mappingproxy.get_item(index).unwrap().extract::<usize>().unwrap(),
            index - 1
        );
    }
}

#[test]
fn iter_mappingproxy_nosegv() {
    let gil = Python::acquire_gil();
    let py = gil.python();
    let mappingproxy = (0..LEN as u64).map(|i| (i, i * 2)).into_py_mappingproxy(py);

    let mut sum = 0;
    for (k, _v) in mappingproxy.iter() {
        let i: u64 = k.extract().unwrap();
        sum += i;
    }
    assert_eq!(sum, 49_999_995_000_000);
}
