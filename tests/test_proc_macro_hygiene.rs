#![no_implicit_prelude]

#[::pyo3::proc_macro::pyclass]
#[derive(::std::clone::Clone)]
pub struct Foo;

#[::pyo3::proc_macro::pyclass]
pub struct Foo2;

#[::pyo3::proc_macro::pyclass(
    name = "ActuallyBar",
    freelist = 8,
    weakref,
    unsendable,
    gc,
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

#[::pyo3::proc_macro::pyproto]
impl ::pyo3::class::gc::PyGCProtocol for Bar {
    fn __traverse__(
        &self,
        visit: ::pyo3::class::gc::PyVisit,
    ) -> ::std::result::Result<(), ::pyo3::class::gc::PyTraverseError> {
        if let ::std::option::Option::Some(obj) = &self.c {
            visit.call(obj)?
        }
        ::std::result::Result::Ok(())
    }

    fn __clear__(&mut self) {
        self.c = ::std::option::Option::None;
    }
}
