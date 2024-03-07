# Calling Python in Rust code

This chapter of the guide documents some ways to interact with Python code from Rust:
 - How to call Python functions
 - How to execute existing Python code

## Interacting with the Python interpreter

To safely interact with the Python interpreter a Rust thread must have a corresponding Python thread state and hold the Global Interpreter Lock (GIL). PyO3 has a `Python<'py>` token which is used to prove that these conditions
are met. It's lifetime `'py` is a central part of PyO3's API.

The `Python<'py>` token serves three purposes:

* It provides global API for the Python interpreter, such as [`py.eval()`][eval] and [`py.import()`][import].
* It can be passed to functions that require a proof of holding the GIL, such as [`Py::clone_ref`][clone_ref].
* Its lifetime `'py` is used to bind many of PyO3's types to the Python interpreter, such as [`Bound<'py, T>`][Bound].

PyO3's types which are bound to the `'py` lifetime, for example `Bound<'py, T>`, all contain a `Python<'py>` token. This means they have full access to the Python interpreter and offer a complete API for interacting with Python objects.

Consult [PyO3's API documentation][obtaining-py] to learn how to acquire one of these tokens.

## Python's memory model

Python memory model differs from Rust's memory model in two key ways:
- There is no concept of ownership; all Python objects are reference counted
- There is no concept of exclusive (`&mut`) references; any reference can mutate a Python object

PyO3's API reflects this by providing smart pointer types, `Py<T>`, `Bound<'py, T>`, and (the very rarely used) `Borrowed<'a, 'py, T>`. These smart pointers all use Python reference counting. See the [subchapter on types](./types.md) for more detail on these types.

### Mutable access to `#[pyclass]` types

The situation is helped by the Global Interpreter Lock (GIL), which ensures that only one thread can use the Python interpreter and its API at the same time, while non-Python operations (system calls and extension code) can unlock the GIL. (See [the section on parallelism](parallelism.md) for how to do that in PyO3.)



The latter two points are the reason why some APIs in PyO3 require the `py:
Python` argument, while others don't.

The PyO3 API for Python objects is written such that instead of requiring a
mutable Rust reference for mutating operations such as
[`PyList::append`][PyList_append], a shared reference (which, in turn, can only
be created through `Python<'_>` with a GIL lifetime) is sufficient.

However, Rust structs wrapped as Python objects (called `pyclass` types) usually
*do* need `&mut` access.  Due to the GIL, PyO3 *can* guarantee thread-safe access
to them, but it cannot statically guarantee uniqueness of `&mut` references once
an object's ownership has been passed to the Python interpreter, ensuring
references is done at runtime using `PyCell`, a scheme very similar to
`std::cell::RefCell`.

[obtaining-py]: {{#PYO3_DOCS_URL}}/pyo3/marker/struct.Python.html#obtaining-a-python-token
