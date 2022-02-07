#![no_implicit_prelude]
#![allow(unused_variables)]

#[crate::pyclass]
#[pyo3(crate = "crate")]
#[derive(::std::clone::Clone)]
pub struct Foo;

#[crate::pyclass]
#[pyo3(crate = "crate")]
pub struct Foo2;

#[crate::pyclass(
    name = "ActuallyBar",
    freelist = 8,
    weakref,
    unsendable,
    subclass,
    extends = crate::types::PyAny,
    module = "Spam"
)]
#[pyo3(crate = "crate")]
pub struct Bar {
    #[pyo3(get, set)]
    a: u8,
    #[pyo3(get, set)]
    b: Foo,
    #[pyo3(get, set)]
    c: ::std::option::Option<crate::Py<Foo2>>,
}

#[crate::pyclass]
#[pyo3(crate = "crate")]
pub enum Enum {
    Var0,
}
