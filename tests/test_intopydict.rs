use pyo3::{
    types::{IntoPyDict, PyDict},
    Python,
    prelude::*
};

pub trait TestTrait<'a> {}

#[derive(IntoPyDict, PartialEq, Debug)]
pub struct Test1 {
    x: u8,
    y: u8,
}

#[derive(IntoPyDict)]
pub struct Test {
    v: Vec<Vec<Test1>>,
    j: Test1,
    h: u8,
}

#[test]
fn test_into_py_dict_derive() {
    let test_struct = Test {
        v: vec![vec![Test1 { x: 9, y: 10 }]],
        j: Test1 { x: 10, y: 11 },
        h: 9,
    };

    Python::with_gil(|py| {
        let py_dict = test_struct.into_py_dict(py);
        let h: u8 = py_dict.get_item("h").unwrap().extract().unwrap();
        assert_eq!(h, 9);
        println!("{:?}", py_dict);
    });
}
