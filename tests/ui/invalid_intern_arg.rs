use pyo3::Python;

fn main() {
    let foo = if true { "foo" } else { "bar" };
    Python::with_gil(|py| py.import(pyo3::intern_bound!(py, foo)).unwrap());
}
