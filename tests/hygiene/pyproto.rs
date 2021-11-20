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

#[::pyo3::pyproto]
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

#[cfg(not(Py_LIMITED_API))]
#[::pyo3::pyproto]
impl ::pyo3::class::PyBufferProtocol for Bar {
    fn bf_getbuffer(
        _s: ::pyo3::PyRefMut<Self>,
        _v: *mut ::pyo3::ffi::Py_buffer,
        _f: ::std::os::raw::c_int,
    ) -> ::pyo3::PyResult<()> {
        ::std::panic!("unimplemented isn't hygienic before 1.50")
    }
    fn bf_releasebuffer(_s: ::pyo3::PyRefMut<Self>, _v: *mut ::pyo3::ffi::Py_buffer) {}
}
