#![no_implicit_prelude]

#[derive(crate::FromPyObject)]
#[pyo3(crate = "crate")]
struct Derive1(#[allow(dead_code)] i32); // newtype case

#[derive(crate::FromPyObject)]
#[pyo3(crate = "crate")]
#[allow(dead_code)]
struct Derive2(i32, i32); // tuple case

#[derive(crate::FromPyObject)]
#[pyo3(crate = "crate")]
#[allow(dead_code)]
struct Derive3 {
    f: i32,
    g: i32,
} // struct case

#[derive(crate::FromPyObject)]
#[pyo3(crate = "crate")]
#[allow(dead_code)]
enum Derive4 {
    A(i32),
    B { f: i32 },
} // enum case

crate::create_exception!(mymodule, CustomError, crate::exceptions::PyException);
crate::import_exception!(socket, gaierror);

#[allow(dead_code)]
fn intern(py: crate::Python<'_>) {
    let _foo = crate::intern!(py, "foo");
    let _bar = crate::intern!(py, stringify!(bar));
}

#[allow(dead_code)]
#[cfg(not(PyPy))]
fn append_to_inittab() {
    #[crate::pymodule]
    #[pyo3(crate = "crate")]
    #[allow(clippy::unnecessary_wraps)]
    fn module_for_inittab(_: crate::Python<'_>, _: &crate::types::PyModule) -> crate::PyResult<()> {
        ::std::result::Result::Ok(())
    }
    crate::append_to_inittab!(module_for_inittab);
}
