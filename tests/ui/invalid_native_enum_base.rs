use pyo3::py_native_enum;

#[py_native_enum(base = "NotABase")]
enum Foo {
    A,
    B,
}

fn main() {}
