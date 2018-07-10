#![feature(proc_macro, specialization)]

extern crate pyo3;

use pyo3::prelude::*;
use pyo3::py::{class, methods};

#[class]
pub struct User {
    name: String,
    age: u32
}

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
fn test_user_derive() {
}

