use pyo3::py_native_enum;

#[derive(pyo3::NativeEnum)]
enum WithTypeParam<T> {
    A,
    B,
    _Phantom(std::marker::PhantomData<T>),
}

#[py_native_enum]
enum WithLifetime<'a> {
    A,
    B,
    _Phantom(&'a ()),
}

fn main() {}
