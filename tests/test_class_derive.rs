#![feature(proc_macro, specialization)]

extern crate pyo3;

use pyo3::prelude::*;
use pyo3::py::class;

#[class]
pub struct User {
    name: String,
    age: u32
}

#[class]
pub struct LinkedListNode<'a> {
    data: u32,
    next: &'a LinkedListNode<'a>,
}

#[test]
fn test_user_derive() {
}

