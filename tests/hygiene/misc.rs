#![no_implicit_prelude]

#[derive(::pyo3::FromPyObject)]
struct Derive1(i32); // newtype case

#[derive(::pyo3::FromPyObject)]
#[allow(dead_code)]
struct Derive2(i32, i32); // tuple case

#[derive(::pyo3::FromPyObject)]
#[allow(dead_code)]
struct Derive3 {
    f: i32,
    g: i32,
} // struct case

#[derive(::pyo3::FromPyObject)]
#[allow(dead_code)]
enum Derive4 {
    A(i32),
    B { f: i32 },
} // enum case

::pyo3::create_exception!(mymodule, CustomError, ::pyo3::exceptions::PyException);
::pyo3::import_exception!(socket, gaierror);
