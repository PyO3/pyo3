#[macro_use] extern crate cpython;

use cpython::{PyObject, PyResult, Python, PyTuple, PyDict};

// Our module is named 'hello', and can be imported using `import hello`.
// This requires that the output binary file is named `hello.so` (or Windows: `hello.pyd`).
// As the output name cannot be configured in cargo (https://github.com/rust-lang/cargo/issues/1970),
// you'll have to rename the output file.
py_module_initializer!(hello, inithello, PyInit_hello, |py, m| {
    m.add(py, "__doc__", "Module documentation string")?;
    m.add(py, "func", py_fn!(py, func(a: &str, b: i32)))?;
    m.add(py, "run", py_fn!(py, run(*args, **kwargs)))?;
    Ok(())
});

// The py_fn!()-macro can translate between Python and Rust values,
// so you can use `&str`, `i32` or `String` in the signature of a function
// callable from Python.
// The first argument of type `Python<'p>` is used to indicate that your
// function may assume that the current thread holds the global interpreter lock.
// Most functions in the `cpython` crate require that you pass this argument.
fn func(_: Python, a: &str, b: i32) -> PyResult<String> {
    Ok(format!("func({}, {})", a, b))
}

fn run(py: Python, args: &PyTuple, kwargs: Option<&PyDict>) -> PyResult<PyObject> {
    println!("Rust says: Hello Python!");
    for arg in args.iter(py) {
        println!("Rust got {}", arg);
    }
    if let Some(kwargs) = kwargs {
        for (key, val) in kwargs.items(py) {
            println!("{} = {}", key, val);
        }
    }
    Ok(py.None())
}

