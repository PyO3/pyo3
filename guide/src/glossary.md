# Glossary

Many of the terms used in this guide are common to the Python or Rust ecosystem, and accordingly can be found in the [Python Glossary](https://docs.python.org/3/glossary.html), [Rust Glossary](https://doc.rust-lang.org/reference/glossary.html), or [Cargo Glossary](https://doc.rust-lang.org/cargo/appendix/glossary.html).

Below are some terms that are particularly relevant to PyO3 and either are not found in the above glossaries or have a specific meaning in the context of PyO3:

attached
  : To call into the Python interpreter from Rust, the Rust thread must have an associated Python thread state which is "attached" to the Python interpreter.
    This is a safety invariant required by the Python C API, which ensures that the Python interpreter can handle exceptions, avoid data races with the garbage collector, etc.
    The [`Python::attach`]({{#PYO3_DOCS_URL}}/pyo3/marker/struct.Python.html#method.attach) method is used to attach the current thread to the Python interpreter and obtain a `Python` token which can be used to call Python APIs.

extension module
  : A Python module which is implemeted using native code (e.g. C, C++ or Rust) instead of Python.
    See also [CPython's documentation on creating extension modules](https://docs.python.org/3/extending/extending.html).

GIL-enabled Python
  : The (current, as of Python 3.14) default build of Python, which depends on the GIL for thread safety.

    This was historically the only thread safety strategy before the introduction of free-threaded Python.

Python token
  : PyO3's [`Python<'py>`]({{#PYO3_DOCS_URL}}/pyo3/marker/struct.Python.html) type, which can only exist when the current thread is [attached](#attached) to the Python interpreter.
    This type is used to call Python APIs and ensures that the safety invariants required by the Python C API are upheld.

smart pointer
  : Pointers which automatically manage the memory they point to.
    In particular this guide refers to [PyO3's smart pointers](types.md#pyo3s-smart-pointers) `Py<T>`, `Bound<'py, T>`, and `Borrowed<'a, 'py, T>` which point to Python objects and use Python reference counting to ensure correct memory management.

    See also the [Rust book's chapter on smart pointers](https://doc.rust-lang.org/book/ch15-00-smart-pointers.html).

wheel
  : A precompiled Python package format.
    Wheels are the standard way to distribute native code in Python packages for common operating systems and Python versions, which avoid the need for users to compile from source.
    The alternative is known as the "source distribution", or "sdist", which requires users to compile the code as part of package installation.

<!-- external-glossary-links
data races: https://docs.python.org/3/glossary.html#term-data-race
free-threading: https://docs.python.org/3/glossary.html#term-free-threading
free-threaded Python: https://docs.python.org/3/glossary.html#term-free-threaded-build
garbage collection: https://docs.python.org/3/glossary.html#term-garbage-collection
GIL: https://docs.python.org/3/glossary.html#term-GIL
global interpreter lock: https://docs.python.org/3/glossary.html#term-global-interpreter-lock
thread state: https://docs.python.org/3/glossary.html#term-thread-state
virtual environment: https://docs.python.org/3/glossary.html#term-virtual-environment
-->
