#[derive(crate::FromPyObject)]
#[pyo3(crate = "crate")]
struct Derive1(i32); // newtype case

#[derive(crate::FromPyObject)]
#[pyo3(crate = "crate")]
struct Derive2(i32, i32); // tuple case

#[derive(crate::FromPyObject)]
#[pyo3(crate = "crate")]
struct Derive3 {
    f: i32,
    #[pyo3(item(42))]
    g: i32,
} // struct case

#[derive(crate::FromPyObject)]
#[pyo3(crate = "crate")]
enum Derive4 {
    A(i32),
    B { f: i32 },
} // enum case

crate::create_exception!(mymodule, CustomError, crate::exceptions::PyException);
crate::import_exception!(socket, gaierror);

fn intern(py: crate::Python<'_>) {
    let _foo = crate::intern!(py, "foo");
    let _bar = crate::intern!(py, stringify!(bar));
}

#[cfg(not(PyPy))]
fn append_to_inittab() {
    #[crate::pymodule]
    #[pyo3(crate = "crate")]
    mod module_for_inittab {}

    crate::append_to_inittab!(module_for_inittab);
}
