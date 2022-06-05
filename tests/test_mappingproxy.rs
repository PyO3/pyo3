use std::collections::HashMap;

use pyo3::prelude::Python;
use pyo3::types::{IntoPyMappingProxy, PyString, PyInt};

const LEN: usize = 10_000_000;

#[test]
fn get_value_from_mappingproxy_of_strings(){
    let gil = Python::acquire_gil();
    let py = gil.python();

    let mut map = HashMap::new();
    map.insert("first key", "first value");
    map.insert("second key", "second value");
    map.insert("third key", "third value");

    let mappingproxy = map.iter().into_py_mappingproxy(py).unwrap();

    assert_eq!(
        map.into_iter().collect::<Vec<(&str, &str)>>(),
        mappingproxy.iter().map(
            |object|
                (
                    object.0.downcast::<PyString>().unwrap().to_str().unwrap(),
                    object.1.downcast::<PyString>().unwrap().to_str().unwrap()
                )
        ).collect::<Vec<(&str, &str)>>()
    );
}

#[test]
fn get_value_from_mappingproxy_of_integers(){
    let gil = Python::acquire_gil();
    let py = gil.python();

    let items: Vec<(usize, usize)> = (1..LEN).map(|i| (i, i - 1)).collect();
    let mappingproxy = items.to_vec().into_py_mappingproxy(py).unwrap();
    assert_eq!(
        items,
        mappingproxy.iter().map(
            |object|
                (
                    object.0.downcast::<PyInt>().unwrap().extract::<usize>().unwrap(),
                    object.1.downcast::<PyInt>().unwrap().extract::<usize>().unwrap()
                )
        ).collect::<Vec<(usize, usize)>>()
    );
    for index in 1..LEN {
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
    let mappingproxy = (0..LEN as u64).map(|i| (i, i * 2)).into_py_mappingproxy(py).unwrap();

    let mut sum = 0;
    for (k, _v) in mappingproxy.iter() {
        let i: u64 = k.extract().unwrap();
        sum += i;
    }
    assert_eq!(sum, 49_999_995_000_000);
}
