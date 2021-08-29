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
