#![cfg(feature = "macros")]

use pyo3::{
    prelude::IntoPyDict,
    types::{IntoPyDict, PyDict},
    Python,
};

pub trait TestTrait<'a> {}

#[derive(IntoPyDict, PartialEq, Debug, Clone)]
pub struct Test1 {
    x: u8,
}

#[derive(IntoPyDict, Clone)]
pub struct Test {
    v: Vec<Vec<Test1>>,
    j: Test1,
    h: u8,
}

#[derive(IntoPyDict)]
pub struct TestGeneric<T: IntoPyDict, U: IntoPyDict> {
    x: T,
    y: TestGenericDouble<T, U>,
}

#[derive(IntoPyDict)]
pub struct TestGenericDouble<T: IntoPyDict, U: IntoPyDict> {
    x: T,
    y: U,
}

#[derive(IntoPyDict)]
pub struct TestVecPrim {
    v: Vec<u8>,
}

#[test]
fn test_into_py_dict_derive() {
    let test_struct = Test {
        v: vec![vec![Test1 { x: 9 }]],
        j: Test1 { x: 10 },
        h: 9,
    };

    let test_generic_struct = TestGeneric {
        x: test_struct.clone(),
        y: TestGenericDouble {
            x: test_struct.clone(),
            y: test_struct.clone(),
        },
    };

    Python::with_gil(|py| {
        let py_dict = test_struct.into_py_dict(py);
        let h: u8 = py_dict.get_item("h").unwrap().extract().unwrap();
        assert_eq!(h, 9);
        assert_eq!(
            format!("{:?}", py_dict),
            "{'v': [[{'x': 9}]], 'j': {'x': 10}, 'h': 9}".to_string()
        );
        let pydict = test_generic_struct.into_py_dict(py);
        assert_eq!(format!("{:?}", pydict), "{'x': {'v': [[{'x': 9}]], 'j': {'x': 10}, 'h': 9}, 'y': {'x': {'v': [[{'x': 9}]], 'j': {'x': 10}, 'h': 9}, 'y': {'v': [[{'x': 9}]], 'j': {'x': 10}, 'h': 9}}}".to_string());
    });
}
