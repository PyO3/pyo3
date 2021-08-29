#![no_implicit_prelude]

#[derive(::pyo3::prelude::FromPyObject)]
struct Derive1(i32); // newtype case

#[derive(::pyo3::prelude::FromPyObject)]
#[allow(dead_code)]
struct Derive2(i32, i32); // tuple case

#[derive(::pyo3::prelude::FromPyObject)]
#[allow(dead_code)]
struct Derive3 {
    f: i32,
    g: i32,
} // struct case

#[derive(::pyo3::prelude::FromPyObject)]
#[allow(dead_code)]
enum Derive4 {
    A(i32),
    B { f: i32 },
} // enum case

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

#[::pyo3::proc_macro::pymethods]
impl Bar {
    #[args(x = "1", "*", _z = "2")]
    fn test(&self, _y: &Bar, _z: i32) {}
    #[staticmethod]
    fn staticmethod() {}
    #[classmethod]
    fn clsmethod(_: &::pyo3::types::PyType) {}
    #[call]
    #[args(args = "*", kwds = "**")]
    fn __call__(
        &self,
        _args: &::pyo3::types::PyTuple,
        _kwds: ::std::option::Option<&::pyo3::types::PyDict>,
    ) -> ::pyo3::PyResult<i32> {
        ::std::panic!("unimplemented isn't hygienic before 1.50")
    }
    #[new]
    fn new(a: u8) -> Self {
        Bar {
            a,
            b: Foo,
            c: ::std::option::Option::None,
        }
    }
    #[getter]
    fn get(&self) -> i32 {
        0
    }
    #[setter]
    fn set(&self, _v: i32) {}
    #[classattr]
    fn class_attr() -> i32 {
        0
    }
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

#[cfg(not(Py_LIMITED_API))]
#[::pyo3::proc_macro::pyproto]
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

#[::pyo3::proc_macro::pyfunction]
fn do_something(x: i32) -> ::pyo3::PyResult<i32> {
    ::std::result::Result::Ok(x)
}

#[::pyo3::proc_macro::pymodule]
fn my_module(_py: ::pyo3::Python, m: &::pyo3::types::PyModule) -> ::pyo3::PyResult<()> {
    m.add_function(::pyo3::wrap_pyfunction!(do_something, m)?)?;
    ::std::result::Result::Ok(())
}

#[test]
fn invoke_wrap_pyfunction() {
    ::pyo3::Python::with_gil(|py| {
        let func = ::pyo3::wrap_pyfunction!(do_something)(py).unwrap();
        ::pyo3::py_run!(py, func, r#"func(5)"#);
    });
}

::pyo3::create_exception!(mymodule, CustomError, ::pyo3::exceptions::PyException);
::pyo3::import_exception!(socket, gaierror);
