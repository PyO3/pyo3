# Using Rust from Python

This chapter of the guide is dedicated to explaining how to wrap Rust code into Python objects.

PyO3 uses Rust's "procedural macros" to provide a powerful yet simple API to denote what Rust code should map into Python objects.

PyO3 can create three types of Python objects:

- Python modules, via the `#[pymodule]` macro
- Python functions, via the `#[pyfunction]` macro
- Python classes, via the `#[pyclass]` macro (plus `#[pymethods]` to define methods for those classes)

The following subchapters go through each of these in turn.
