# Supporting Free-Threaded CPython

CPython 3.13 introduces an experimental build of CPython that does not rely on
the global interpreter lock (often referred to as the GIL) for thread safety. As
of version 0.23, PyO3 also has preliminary support for building rust extensions
for the free-threaded Python build and support for calling into free-threaded
Python from Rust.

The main benefit for supporting free-threaded Python is that it is no longer
necessary to rely on rust parallelism to achieve concurrent speedups using
PyO3. Instead, you can parallelise in Python using the
[`threading`](https://docs.python.org/3/library/threading.html) module, and
still expect to see multicore speedups by exploiting threaded concurrency in
Python, without any need to release the GIL. If you have ever needed to use
`multiprocessing` to achieve a speedup for some algorithm written in Python,
free-threading will likely allow the use of Python threads instead for the same
workflow.

PyO3's support for free-threaded Python will enable authoring native Python
extensions that are thread-safe by construction, with much stronger safety
guarantees than C extensions. Our goal is to enable ["fearless
concurrency"](https://doc.rust-lang.org/book/ch16-00-concurrency.html) in the
native Python runtime by building on the rust `Send` and `Sync` traits.

If you want more background on free-threaded Python in general, see the [what's
new](https://docs.python.org/3.13/whatsnew/3.13.html#whatsnew313-free-threaded-cpython)
entry in the CPython docs, the [HOWTO
guide](https://docs.python.org/3.13/howto/free-threading-extensions.html#freethreading-extensions-howto)
for porting C extensions, and [PEP 703](https://peps.python.org/pep-0703/),
which provides the technical background for the free-threading implementation in
CPython.

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
Python **thread state**. See [PEP
703](https://peps.python.org/pep-0703/#thread-states) for more background about
Python thread states and status.

In order to use the CPython C API in both the GIL-enabled and free-threaded
builds of CPython, the thread calling into the C API must own an attached Python
thread state. In the GIL-enabled build the thread that holds the GIL by
definition is attached to a valid Python thread state, and therefore only one
thread at a time can call into the C API.

What a thread releases the GIL, the Python thread state owned by that thread is
detached from the interpreter runtime, and it is not valid to call into the
CPython C API.

In the free-threaded build, more than one thread can simultaneously call into
the C API, but any thread that does so must still have a reference to a valid
attached thread state. The CPython runtime also assumes it is responsible for
creating and destroying threads, so it is necessary to detach from the runtime
before creating any native threads outside of the CPython runtime. In the
GIL-enabled build, this corresponds to dropping the GIL with an `allow_threads`
call.

In the GIL-enabled build, releasing the GIL allows other threads to
proceed. This is no longer necessary in the free-threaded build, but you should
still release the GIL when doing long-running tasks that do not require the
CPython runtime, since releasing the GIL unblocks running the Python garbage
collector and freeing unused memory.

## Runtime panics for multithreaded access of mutable `pyclass` instances

If you wrote code that makes strong assumptions about the GIL protecting shared
mutable state, it may not currently be straightforward to support free-threaded
Python without the risk of runtime mutable borrow panics. PyO3 does not lock
access to Python state, so if more than one thread tries to access a Python
object that has already been mutably borrowed, only runtime checking enforces
safety around mutably aliased data owned by the Python interpreter. We believe
that it would require adding an `unsafe impl` for `Send` or `Sync` to trigger
this behavior. Please report any issues related to runtime borrow checker errors
on mutable pyclass implementations that do not make strong assumptions about the
GIL.

It was always possible to generate panics like this in PyO3 in code that
releases the GIL with `allow_threads` (see [the docs on interior
mutability](./class.md#bound-and-interior-mutability),) but now in free-threaded
Python there are more opportunities to trigger these panics because there is no
GIL.

We plan to allow user-selectable semantics for for mutable pyclass definitions in
PyO3 0.24, allowing some form of opt-in locking to emulate the GIL if
that is needed.

## `GILProtected` is not exposed

`GILProtected` is a PyO3 type that allows mutable access to static data by
leveraging the GIL to lock concurrent access from other threads. In
free-threaded Python there is no GIL, so you will need to replace this type with
some other form of locking. In many cases, `std::sync::Atomic` or
`std::sync::Mutex` will be sufficient. If the locks do not guard the execution
of arbitrary Python code or use of the CPython C API then conditional
compilation is likely unnecessary since `GILProtected` was not needed in the
first place.

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
builds and mutexes otherwise. Python 3.13 introduces `PyMutex`, which releases
the GIL while the lock is held, so that is another option if you only need to
support newer Python versions.
