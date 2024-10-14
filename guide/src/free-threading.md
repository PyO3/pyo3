# Supporting Free-Threaded CPython

CPython 3.13 introduces an experimental "free-threaded" build of CPython that
does not rely on the [global interpreter
lock](https://docs.python.org/3/glossary.html#term-global-interpreter-lock)
(often referred to as the GIL) for thread safety. As of version 0.23, PyO3 also
has preliminary support for building rust extensions for the free-threaded
Python build and support for calling into free-threaded Python from Rust.

If you want more background on free-threaded Python in general, see the [what's
new](https://docs.python.org/3.13/whatsnew/3.13.html#whatsnew313-free-threaded-cpython)
entry in the CPython docs, the [HOWTO
guide](https://docs.python.org/3.13/howto/free-threading-extensions.html#freethreading-extensions-howto)
for porting C extensions, and [PEP 703](https://peps.python.org/pep-0703/),
which provides the technical background for the free-threading implementation in
CPython.

The main benefit for supporting free-threaded Python is that both pure Python
code and code interacting with Python via the CPython C API can run
simultaneously on multiple OS threads.  This means it's much more
straightforward to achieve multithreaded parallelism in the Python layer, using
the `threading` module, without any need to ensure the GIL is released during
compute-heavy tasks, or any worries about [Amdahl's
law](https://en.wikipedia.org/wiki/Amdahl%27s_law) limiting parallel
performance. If you have ever needed to use `multiprocessing` to achieve a
speedup for some Python code, free-threading will likely allow the use of Python
threads instead for the same workflow.

PyO3's support for free-threaded Python will enable authoring native Python
extensions that are thread-safe by construction, with much stronger safety
guarantees than C extensions. Our goal is to enable ["fearless
concurrency"](https://doc.rust-lang.org/book/ch16-00-concurrency.html) in the
native Python runtime by building on the rust `Send` and `Sync` traits.

This document provides advice for porting rust code using PyO3 to run under
free-threaded Python. While many simple PyO3 uses, like defining an immutable
Python class, will likely work "out of the box", there are currently some
limitations.

## Many symbols exposed by PyO3 have `GIL` in the name

We are aware that there are some naming issues in the PyO3 API that make it
awkward to think about a runtime environment where there is no GIL. We plan to
change the names of these types to de-emphasize the role of the GIL in future
versions of PyO3, but for now you should remember that the use of the term `GIL`
in functions and types like `with_gil` and `GILOnceCell` is historical.

Instead, you can think about whether or not a rust thread is attached to a
Python interpreter runtime. See [PEP
703](https://peps.python.org/pep-0703/#thread-states) for more background about
how threads can be attached and detached from the interpreter runtime, in a
manner analagous to releasing and acquiring the GIL in the GIL-enabled build.

Calling into the CPython C API is only legal when an OS thread is explicitly
attached to the interpreter runtime. In the GIL-enabled build, this happens when
the GIL is acquired. In the free-threaded build there is no GIL, but the same C
macros that release or acquire the GIL in the GIL-enabled build instead ask the
interpreter to attach the thread to the Python runtime, and there can be many
threads simultaneously attached.

The main reason for attaching to the Python runtime is to interact with Python
objects or call into the CPython C API. To interact with the Python runtime, the
thread must register itself by attaching to the interpreter runtime.

In the GIL-enabled build, releasing the GIL allows other threads to
proceed. This is no longer necessary in the free-threaded build, but you should
still detach from the interpreter runtime using [`Python::allow_threads`] when
doing long-running tasks that do not require the CPython runtime, since
detaching from the runtime allows the Python garbage collector to run, freeing
unused memory.

## Runtime panics for multithreaded access of mutable `pyclass` instances

If you wrote code that makes strong assumptions about the GIL protecting shared
mutable state, it may not currently be straightforward to support free-threaded
Python without the risk of runtime mutable borrow panics. PyO3 does not lock
access to Python state, so if more than one thread tries to access a Python
object that has already been mutably borrowed, only runtime checking enforces
safety around mutably aliased rust variables the Python interpreter can
access. We believe that it would require adding an `unsafe impl` for `Send` or
`Sync` to trigger this behavior in code using PyO3. Please report any issues
related to runtime borrow checker errors on mutable pyclass implementations that
do not make strong assumptions about the GIL.

It was always possible to generate panics like this in PyO3 in code that
releases the GIL with `allow_threads` (see [the docs on interior
mutability](./class.md#bound-and-interior-mutability),) but now in free-threaded
Python there are more opportunities to trigger these panics because there is no
GIL.

We plan to allow user-selectable semantics for mutable pyclass definitions in
PyO3 0.24, allowing some form of opt-in locking to emulate the GIL if that is
needed.

## `GILProtected` is not exposed

`GILProtected` is a PyO3 type that allows mutable access to static data by
leveraging the GIL to lock concurrent access from other threads. In
free-threaded Python there is no GIL, so you will need to replace this type with
some other form of locking. In many cases, a type from `std::sync::Atomic` or
a `std::sync::Mutex` will be sufficient.

Before:

```rust
# fn main() {
# #[cfg(not(Py_GIL_DISABLED))] {
# use pyo3::prelude::*;
use pyo3::sync::GILProtected;
use pyo3::types::{PyDict, PyNone};
use std::cell::RefCell;

static OBJECTS: GILProtected<RefCell<Vec<Py<PyDict>>>> =
    GILProtected::new(RefCell::new(Vec::new()));

Python::with_gil(|py| {
    // stand-in for something that executes arbitrary Python code
    let d = PyDict::new(py);
    d.set_item(PyNone::get(py), PyNone::get(py)).unwrap();
    OBJECTS.get(py).borrow_mut().push(d.unbind());
});
# }}
```

After:

```rust
# use pyo3::prelude::*;
# fn main() {
use pyo3::types::{PyDict, PyNone};
use std::sync::Mutex;

static OBJECTS: Mutex<Vec<Py<PyDict>>> = Mutex::new(Vec::new());

Python::with_gil(|py| {
    // stand-in for something that executes arbitrary Python code
    let d = PyDict::new(py);
    d.set_item(PyNone::get(py), PyNone::get(py)).unwrap();
    // we're not executing Python code while holding the lock, so GILProtected
    // was never needed
    OBJECTS.lock().unwrap().push(d.unbind());
});
# }
```

If you are executing arbitrary Python code while holding the lock, then you will
need to use conditional compilation to use `GILProtected` on GIL-enabled Python
builds and mutexes otherwise. If your use of `GILProtected` does not guard the
execution of arbitrary Python code or use of the CPython C API, then conditional
compilation is likely unnecessary since `GILProtected` was not needed in the
first place and instead rust mutexes or atomics should be preferred. Python 3.13
introduces `PyMutex`, which releases the GIL while the lock is held, so that is
another option if you only need to support newer Python versions.
