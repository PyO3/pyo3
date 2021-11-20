#![no_implicit_prelude]
#![allow(unused_variables)]

#[::pyo3::pyclass]
#[derive(::std::clone::Clone)]
pub struct Foo;

#[::pyo3::pyclass]
pub struct Foo2;

#[::pyo3::pyclass(
    name = "ActuallyBar",
    freelist = 8,
    weakref,
    unsendable,
    subclass,
    extends = ::pyo3::types::PyAny,
    module = "Spam"
)]
pub struct Bar {
    #[pyo3(get, set)]
    a: u8,
    #[pyo3(get, set)]
    b: Foo,
    #[pyo3(get, set)]
    c: ::std::option::Option<::pyo3::Py<Foo2>>,
}
