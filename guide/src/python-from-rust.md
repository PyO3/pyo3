# Calling Python in Rust code

This chapter of the guide documents some ways to interact with Python code from Rust.

Below is an introduction to the `'py` lifetime and some general remarks about how PyO3's API reasons about Python code.

The subchapters also cover the following topics:

- Python object types available in PyO3's API
- How to work with Python exceptions
- How to call Python functions
- How to execute existing Python code

## The `'py` lifetime

To safely interact with the Python interpreter a Rust thread must be [attached] to the Python interpreter.
PyO3 has a `Python<'py>` token that is used to prove that these conditions are met.
Its lifetime `'py` is a central part of PyO3's API.

The `Python<'py>` token serves three purposes:

- It provides global APIs for the Python interpreter, such as [`py.eval()`][eval] and [`py.import()`][import].
- It can be passed to functions that require a proof of attachment, such as [`Py::clone_ref`][clone_ref].
- Its lifetime `'py` is used to bind many of PyO3's types to the Python interpreter, such as [`Bound<'py, T>`][Bound].

PyO3's types that are bound to the `'py` lifetime, for example `Bound<'py, T>`, all contain a `Python<'py>` token.
This means they have full access to the Python interpreter and offer a complete API for interacting with Python objects.

Consult [PyO3's API documentation][obtaining-py] to learn how to acquire one of these tokens.

### The Global Interpreter Lock

Prior to the introduction of free-threaded Python (first available in 3.13, fully supported in 3.14), the Python interpreter was made thread-safe by the [global interpreter lock].
This ensured that only one Python thread can use the Python interpreter and its API at the same time.
Historically, Rust code was able to use the GIL as a synchronization guarantee, but the introduction of free-threaded Python removed this possibility.

The [`pyo3::sync`] module offers synchronization tools which abstract over both Python builds.

To enable any parallelism on the GIL-enabled build, and best throughput on the free-threaded build, non-Python operations (system calls and native Rust code) should consider detaching from the Python interpreter to allow other work to proceed.
See [the section on parallelism](parallelism.md) for how to do that using PyO3's API.

## Python's memory model

Python's memory model differs from Rust's memory model in two key ways:

- There is no concept of ownership; all Python objects are shared and usually implemented via reference counting
- There is no concept of exclusive (`&mut`) references; any reference can mutate a Python object

PyO3's API reflects this by providing [smart pointer][smart-pointers] types, `Py<T>`, `Bound<'py, T>`, and (the very rarely used) `Borrowed<'a, 'py, T>`.
These smart pointers all use Python reference counting.
See the [subchapter on types](./types.md) for more detail on these types.

Because of the lack of exclusive `&mut` references, PyO3's APIs for Python objects, for example [`PyListMethods::append`], use shared references.
This is safe because Python objects have internal mechanisms to prevent data races (as of time of writing, the Python GIL).

[attached]: https://docs.python.org/3.14/glossary.html#term-attached-thread-state
[global interpreter lock]: https://docs.python.org/3/c-api/init.html#thread-state-and-the-global-interpreter-lock
[smart-pointers]: https://doc.rust-lang.org/book/ch15-00-smart-pointers.html
[obtaining-py]: {{#PYO3_DOCS_URL}}/pyo3/marker/struct.Python.html#obtaining-a-python-token
[`pyo3::sync`]: {{#PYO3_DOCS_URL}}/pyo3/sync/index.html
[eval]: {{#PYO3_DOCS_URL}}/pyo3/marker/struct.Python.html#method.eval
[import]: {{#PYO3_DOCS_URL}}/pyo3/marker/struct.Python.html#method.import
[clone_ref]: {{#PYO3_DOCS_URL}}/pyo3/prelude/struct.Py.html#method.clone_ref
[Bound]: {{#PYO3_DOCS_URL}}/pyo3/struct.Bound.html
[`PyListMethods::append`]: {{#PYO3_DOCS_URL}}/pyo3/types/trait.PyListMethods.html#tymethod.append
