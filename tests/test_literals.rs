#![cfg(feature = "macros")]

use pyo3::prelude::*;
use pyo3::{py_dict, py_run, py_tuple};

#[test]
fn test_dict_literal() {
    let gil = Python::acquire_gil();
    let py = gil.python();

    let dict = py_dict!(py => {"key": "value"}).expect("failed to create dict");
    assert_eq!(
        "value",
        dict.get_item("key").unwrap().extract::<String>().unwrap()
    );

    let value = "value";
    let multi_elem_dict =
        py_dict!(py => {"key1": value, 143: "abcde"}).expect("failed to create dict");
    assert_eq!(
        "value",
        multi_elem_dict
            .get_item("key1")
            .unwrap()
            .extract::<&str>()
            .unwrap()
    );
    assert_eq!(
        "abcde",
        multi_elem_dict
            .get_item(143)
            .unwrap()
            .extract::<&str>()
            .unwrap()
    );

    let keys = &["key1", "key2"];

    let expr_dict = py_dict!(py => {
        keys[0]: "value1",
        keys[1]: "value2",
        3-7: py_tuple!(py, ("elem1", "elem2", 3)),
        "KeY".to_lowercase(): 100 * 2,
    })
    .expect("failed to create dict");

    py_run!(
        py,
        expr_dict,
        "assert expr_dict == {'key1': 'value1', 'key2': 'value2', -4: ('elem1', 'elem2', 3), 'key': 200}"
    );
}
