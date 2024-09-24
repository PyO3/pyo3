# Supporting Free-Threaded CPython

CPython 3.13 introduces an experimental build of CPython that does not rely on
the global interpreter lock for thread safety. As of version 0.23, PyO3 also has
preliminary support for building rust extensions for the free-threaded Python
build and support for calling into free-threaded Python from Rust.

The main benefit for supporting free-threaded Python is that it is no longer
necessary to rely on rust parallelism to achieve concurrent speedups using
PyO3. Instead, you can parallelise in Python using the
[`threading`](https://docs.python.org/3/library/threading.html) module, and
still expect to see see multicore speedups by exploiting threaded concurrency in
Python, without any need to release the GIL. If you have ever needed to use
`multiprocessing` to achieve a speedup for some algorithm written in Python,
free-threading will likely allow the use of Python threads instead for the same
workflow.

If you want more background on free-threaded Python in general, see the [what's
new](https://docs.python.org/3.13/whatsnew/3.13.html#whatsnew313-free-threaded-cpython)
entry in the CPython docs, the [HOWTO
guide](https://docs.python.org/3.13/howto/free-threading-extensions.html#freethreading-extensions-howto)
for porting C extensions, and [PEP 703](https://peps.python.org/pep-0703/),
which provides the technical background for the free-threading implementation in
CPython.

This document provides advice for porting rust code using PyO3 to run under
free-threaded Python. While many simple PyO3 uses, like defining an immutable
python class, will likely work "out of the box", there are currently some
limitations. 

## Many symbols exposed by PyO3 have `GIL` in the name

We are aware that there are some naming issues in the PyO3 API that make it
awkward to work in an environment where there is no GIL. We plan to change the
names of these types to deemphasize the role of the GIL in future versions of
PyO3, but for now you should remember that the use of the term `GIL` in
functions and types like `with_gil` and `GILOnceCell` is historical.

Instead, you can think about whether or not you a rust scope has access to a
Python **thread state** in `ATTACHED` status. See [PEP
703](https://peps.python.org/pep-0703/#thread-states) for more background about
Python thread states and status. In order to use the CPython C API in both the
GIL-enabled and free-threaded builds of CPython, you must own an attached
Python thread state. The `with_gil` function sets this up and releases the
thread state after the closure passed to `with_gil` finishes. Similarly, in both
the GIL-enabled and free-threaded build, you must use `allow_threads` in
order to use rust threads. Both of `with_gil` and `allow_threads` tell CPython
to put the Python thread state into `DETACHED` status. In the GIL-enabled build,
this is equivalent to releasing the GIL. In the free-threaded build, this unblocks
CPython from triggering a stop-the-world for a garbage collection pass.

## Runtime panics for multithreaded access of mutable `pyclass` instances

If you wrote code that makes strong assumptions about the GIL protecting shared
mutable state, it may not currently be straightforward to support free-threaded
Python without the risk of runtime mutable borrow panics. PyO3 does not lock
access to python state, so if more than one thread tries to access a python
object that has already been mutably borrowed, only runtime checking enforces
safety around mutably aliased data owned by the Python interpreter.

It was always possible to generate panics like this in PyO3 in code that
releases the GIL with `allow_threads`, but now in free-threaded python it's much
easier to trigger these panics because there is no GIL.

We will allow user-selectable semantics for for mutable pyclass definitions in
PyO3 0.24, allowing some form of opt-in locking to emulate the GIL if
that is needed.

## `GILProtected` is not exposed

`GILProtected` is a PyO3 type that allows mutable access to static data by
leveraging the GIL to lock concurrent access from other threads. In
free-threaded python there is no GIL, so you will need to replace this type with
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
    // stand-in for something that executes arbitrary python code
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
    // stand-in for something that executes arbitrary python code
    let d = PyDict::new(py);
    d.set_item(PyNone::get(py), PyNone::get(py)).unwrap();
    // we're not executing python code while holding the lock, so GILProtected
    // was never needed
    OBJECTS.lock().unwrap().push(d.unbind());
});
# }
```

If you are executing arbitrary Python code while holding the lock, then you will
need to use conditional compilation to use `GILProtected` on GIL-enabled python
builds and mutexes otherwise. Python 3.13 introduces `PyMutex`, which releases
the GIL while the lock is held, so that is another option if you only need to
support newer Python versions.
