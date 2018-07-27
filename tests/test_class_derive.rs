#![feature(proc_macro, specialization)]

extern crate pyo3;

use pyo3::prelude::*;
use pyo3::py::{class, methods};

#[macro_use]
mod common;

#[class]
pub struct LinkedListNode<'a> {
    data: u32,
    next: Option<&'a LinkedListNode<'a>>,
}

#[methods]
impl<'a> LinkedListNode<'a> {
    #[new]
    fn __new__(obj: &PyRawObject, data: u32) -> PyResult<()> {
        obj.init(|t| LinkedListNode { data, next: None })
    }
}

#[test]
fn test_class_generic_lifetime_derive() {
    let gil = Python::acquire_gil();
    let py = gil.python();
    let typeobj = py.get_type::<LinkedListNode>();
    assert!(typeobj.call(NoArgs, NoArgs).is_err());

    py_assert!(py, typeobj, "typeobj.__name__ == 'LinkedListNode'");
}

