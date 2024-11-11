# Supporting Free-Threaded CPython

CPython 3.13 introduces an experimental "free-threaded" build of CPython that
does not rely on the [global interpreter
lock](https://docs.python.org/3/glossary.html#term-global-interpreter-lock)
(often referred to as the GIL) for thread safety. As of version 0.23, PyO3 also
has preliminary support for building Rust extensions for the free-threaded
Python build and support for calling into free-threaded Python from Rust.

If you want more background on free-threaded Python in general, see the [what's
new](https://docs.python.org/3.13/whatsnew/3.13.html#whatsnew313-free-threaded-cpython)
entry in the CPython docs, the [HOWTO
guide](https://docs.python.org/3.13/howto/free-threading-extensions.html#freethreading-extensions-howto)
for porting C extensions, and [PEP 703](https://peps.python.org/pep-0703/),
which provides the technical background for the free-threading implementation in
CPython.

In the GIL-enabled build, the global interpreter lock serializes access to the
Python runtime. The GIL is therefore a fundamental limitation to parallel
scaling of multithreaded Python workflows, due to [Amdahl's
law](https://en.wikipedia.org/wiki/Amdahl%27s_law), because any time spent
executing a parallel processing task on only one execution context fundamentally
cannot be sped up using parallelism.

The free-threaded build removes this limit on multithreaded Python scaling. This
means it's much more straightforward to achieve parallelism using the Python
[`threading`] module. If you
have ever needed to use
[`multiprocessing`](https://docs.python.org/3/library/multiprocessing.html) to
achieve a parallel speedup for some Python code, free-threading will likely
allow the use of Python threads instead for the same workflow.

PyO3's support for free-threaded Python will enable authoring native Python
extensions that are thread-safe by construction, with much stronger safety
guarantees than C extensions. Our goal is to enable ["fearless
concurrency"](https://doc.rust-lang.org/book/ch16-00-concurrency.html) in the
native Python runtime by building on the Rust [`Send` and
`Sync`](https://doc.rust-lang.org/nomicon/send-and-sync.html) traits.

This document provides advice for porting Rust code using PyO3 to run under
free-threaded Python.

## Supporting free-threaded Python with PyO3

Many simple uses of PyO3, like exposing bindings for a "pure" Rust function
with no side-effects or defining an immutable Python class, will likely work
"out of the box" on the free-threaded build. All that will be necessary is to
annotate Python modules declared by rust code in your project to declare that
they support free-threaded Python, for example by declaring the module with
`#[pymodule(gil_used = false)]`.

At a low-level, annotating a module sets the `Py_MOD_GIL` slot on modules
defined by an extension to `Py_MOD_GIL_NOT_USED`, which allows the interpreter
to see at runtime that the author of the extension thinks the extension is
thread-safe. You should only do this if you know that your extension is
thread-safe. Because of Rust's guarantees, this is already true for many
extensions, however see below for more discussion about how to evaluate the
thread safety of existing Rust extensions and how to think about the PyO3 API
using a Python runtime with no GIL.

If you do not explicitly mark that modules are thread-safe, the Python
interpreter will re-enable the GIL at runtime while importing your module and
print a `RuntimeWarning` with a message containing the name of the module
causing it to re-enable the GIL. You can force the GIL to remain disabled by
setting the `PYTHON_GIL=0` as an environment variable or passing `-Xgil=0` when
starting Python (`0` means the GIL is turned off).

If you are sure that all data structures exposed in a `PyModule` are
thread-safe, then pass `gil_used = false` as a parameter to the
`pymodule` procedural macro declaring the module or call
`PyModule::gil_used` on a `PyModule` instance.  For example:

```rust
use pyo3::prelude::*;

/// This module supports free-threaded Python
#[pymodule(gil_used = false)]
fn my_extension(m: &Bound<'_, PyModule>) -> PyResult<()> {
    // add members to the module that you know are thread-safe
    Ok(())
}
```

Or for a module that is set up without using the `pymodule` macro:

```rust
use pyo3::prelude::*;

# #[allow(dead_code)]
fn register_child_module(parent_module: &Bound<'_, PyModule>) -> PyResult<()> {
    let child_module = PyModule::new(parent_module.py(), "child_module")?;
    child_module.gil_used(false)?;
    parent_module.add_submodule(&child_module)
}

```

For now you must explicitly opt in to free-threading support by annotating
modules defined in your extension. In a future version of `PyO3`, we plan to
make `gil_used = false` the default.

See the
[`string-sum`](https://github.com/PyO3/pyo3/tree/main/pyo3-ffi/examples/string-sum)
example for how to declare free-threaded support using raw FFI calls for modules
using single-phase initialization and the
[`sequential`](https://github.com/PyO3/pyo3/tree/main/pyo3-ffi/examples/sequential)
example for modules using multi-phase initialization.

## Special considerations for the free-threaded build

The free-threaded interpreter does not have a GIL, and this can make interacting
with the PyO3 API confusing, since the API was originally designed around strong
assumptions about the GIL providing locking.  Additionally, since the GIL
provided locking for operations on Python objects, many existing extensions that
provide mutable data structures relied on the GIL to make interior mutability
thread-safe.

Working with PyO3 under the free-threaded interpreter therefore requires some
additional care and mental overhead compared with a GIL-enabled interpreter. We
discuss how to handle this below.

### Many symbols exposed by PyO3 have `GIL` in the name

We are aware that there are some naming issues in the PyO3 API that make it
awkward to think about a runtime environment where there is no GIL. We plan to
change the names of these types to de-emphasize the role of the GIL in future
versions of PyO3, but for now you should remember that the use of the term `GIL`
in functions and types like [`Python::with_gil`] and [`GILOnceCell`] is
historical.

Instead, you should think about whether or not a Rust thread is attached to a
Python interpreter runtime. Calling into the CPython C API is only legal when an
OS thread is explicitly attached to the interpreter runtime. In the GIL-enabled
build, this happens when the GIL is acquired. In the free-threaded build there
is no GIL, but the same C macros that release or acquire the GIL in the
GIL-enabled build instead ask the interpreter to attach the thread to the Python
runtime, and there can be many threads simultaneously attached. See [PEP
703](https://peps.python.org/pep-0703/#thread-states) for more background about
how threads can be attached and detached from the interpreter runtime, in a
manner analagous to releasing and acquiring the GIL in the GIL-enabled build.

In the GIL-enabled build, PyO3 uses the [`Python<'py>`] type and the `'py`
lifetime to signify that the global interpreter lock is held. In the
freethreaded build, holding a `'py` lifetime means only that the thread is
currently attached to the Python interpreter -- other threads can be
simultaneously interacting with the interpreter.

The main reason for obtaining a `'py` lifetime is to interact with Python
objects or call into the CPython C API. If you are not yet attached to the
Python runtime, you can register a thread using the [`Python::with_gil`]
function. Threads created via the Python [`threading`] module do not not need to
do this, but all other OS threads that interact with the Python runtime must
explicitly attach using `with_gil` and obtain a `'py` liftime.

Since there is no GIL in the free-threaded build, releasing the GIL for
long-running tasks is no longer necessary to ensure other threads run, but you
should still detach from the interpreter runtime using [`Python::allow_threads`]
when doing long-running tasks that do not require the CPython runtime. The
garbage collector can only run if all threads are detached from the runtime (in
a stop-the-world state), so detaching from the runtime allows freeing unused
memory.

### Exceptions and panics for multithreaded access of mutable `pyclass` instances

Data attached to `pyclass` instances is protected from concurrent access by a
`RefCell`-like pattern of runtime borrow checking. Like a `RefCell`, PyO3 will
raise exceptions (or in some cases panic) to enforce exclusive access for
mutable borrows. It was always possible to generate panics like this in PyO3 in
code that releases the GIL with [`Python::allow_threads`] or calling a python
method accepting `&self` from a `&mut self` (see [the docs on interior
mutability](./class.md#bound-and-interior-mutability),) but now in free-threaded
Python there are more opportunities to trigger these panics from Python because
there is no GIL to lock concurrent access to mutably borrowed data from Python.

The most straightforward way to trigger this problem to use the Python
[`threading`] module to simultaneously call a rust function that mutably borrows a
[`pyclass`]({{#PYO3_DOCS_URL}}/pyo3/attr.pyclass.html). For example,
consider the following implementation:

```rust
# use pyo3::prelude::*;
# fn main() {
#[pyclass]
#[derive(Default)]
struct ThreadIter {
    count: usize,
}

#[pymethods]
impl ThreadIter {
    #[new]
    pub fn new() -> Self {
        Default::default()
    }

    fn __next__(&mut self, py: Python<'_>) -> usize {
        self.count += 1;
        self.count
    }
}
# }
```

And then if we do something like this in Python:

```python
import concurrent.futures
from my_module import ThreadIter

i = ThreadIter()

def increment():
    next(i)

with concurrent.futures.ThreadPoolExecutor(max_workers=16) as tpe:
    futures = [tpe.submit(increment) for _ in range(100)]
    [f.result() for f in futures]
```

We will see an exception:

```text
Traceback (most recent call last)
  File "example.py", line 5, in <module>
    next(i)
RuntimeError: Already borrowed
```

We plan to allow user-selectable semantics for mutable pyclass definitions in
PyO3 0.24, allowing some form of opt-in locking to emulate the GIL if that is
needed. For now you should explicitly add locking, possibly using conditional
compilation or using the critical section API to avoid creating deadlocks with
the GIL.

## Thread-safe single initialization

Until version 0.23, PyO3 provided only [`GILOnceCell`] to enable deadlock-free
single initialization of data in contexts that might execute arbitrary Python
code. While we have updated [`GILOnceCell`] to avoid thread safety issues
triggered only under the free-threaded build, the design of [`GILOnceCell`] is
inherently thread-unsafe, in a manner that can be problematic even in the
GIL-enabled build.

If, for example, the function executed by [`GILOnceCell`] releases the GIL or
calls code that releases the GIL, then it is possible for multiple threads to
race to initialize the cell. While the cell will only ever be intialized
once, it can be problematic in some contexts that [`GILOnceCell`] does not block
like the standard library [`OnceLock`].

In cases where the initialization function must run exactly once, you can bring
the [`OnceExt`] or [`OnceLockExt`] traits into scope. The [`OnceExt`] trait adds
[`OnceExt::call_once_py_attached`] and [`OnceExt::call_once_force_py_attached`]
functions to the api of `std::sync::Once`, enabling use of [`Once`] in contexts
where the GIL is held. Similarly, [`OnceLockExt`] adds
[`OnceLockExt::get_or_init_py_attached`]. These functions are analogous to
[`Once::call_once`], [`Once::call_once_force`], and [`OnceLock::get_or_init`] except
they accept a [`Python<'py>`] token in addition to an `FnOnce`. All of these
functions release the GIL and re-acquire it before executing the function,
avoiding deadlocks with the GIL that are possible without using the PyO3
extension traits. Here is an example of how to use [`OnceExt`] to
enable single-initialization of a runtime cache holding a `Py<PyDict>`.

```rust
# fn main() {
# use pyo3::prelude::*;
use std::sync::Once;
use pyo3::sync::OnceExt;
use pyo3::types::PyDict;

struct RuntimeCache {
    once: Once,
    cache: Option<Py<PyDict>>
}

let mut cache = RuntimeCache {
    once: Once::new(),
    cache: None
};

Python::with_gil(|py| {
    // guaranteed to be called once and only once
    cache.once.call_once_py_attached(py, || {
        cache.cache = Some(PyDict::new(py).unbind());
    });
});
# }
```

### `GILProtected` is not exposed

[`GILProtected`] is a PyO3 type that allows mutable access to static data by
leveraging the GIL to lock concurrent access from other threads. In
free-threaded Python there is no GIL, so you will need to replace this type with
some other form of locking. In many cases, a type from
[`std::sync::atomic`](https://doc.rust-lang.org/std/sync/atomic/) or a
[`std::sync::Mutex`](https://doc.rust-lang.org/std/sync/struct.Mutex.html) will
be sufficient.

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
    // as with any `Mutex` usage, lock the mutex for as little time as possible
    // in this case, we do it just while pushing into the `Vec`
    OBJECTS.lock().unwrap().push(d.unbind());
});
# }
```

If you are executing arbitrary Python code while holding the lock, then you will
need to use conditional compilation to use [`GILProtected`] on GIL-enabled Python
builds and mutexes otherwise. If your use of [`GILProtected`] does not guard the
execution of arbitrary Python code or use of the CPython C API, then conditional
compilation is likely unnecessary since [`GILProtected`] was not needed in the
first place and instead Rust mutexes or atomics should be preferred. Python 3.13
introduces [`PyMutex`](https://docs.python.org/3/c-api/init.html#c.PyMutex),
which releases the GIL while the waiting for the lock, so that is another option
if you only need to support newer Python versions.

[`GILOnceCell`]: {{#PYO3_DOCS_URL}}/pyo3/sync/struct.GILOnceCell.html
[`GILProtected`]: https://docs.rs/pyo3/0.22/pyo3/sync/struct.GILProtected.html
[`Once`]: https://doc.rust-lang.org/stable/std/sync/struct.Once.html
[`Once::call_once`]: https://doc.rust-lang.org/stable/std/sync/struct.Once.html#tymethod.call_once
[`Once::call_once_force`]: https://doc.rust-lang.org/stable/std/sync/struct.Once.html#tymethod.call_once_force
[`OnceExt`]: {{#PYO3_DOCS_URL}}/pyo3/sync/trait.OnceExt.html
[`OnceExt::call_once_py_attached`]: {{#PYO3_DOCS_URL}}/pyo3/sync/trait.OnceExt.html#tymethod.call_once_py_attached
[`OnceExt::call_once_force_py_attached`]: {{#PYO3_DOCS_URL}}/pyo3/sync/trait.OnceExt.html#tymethod.call_once_force_py_attached
[`OnceLockExt`]: {{#PYO3_DOCS_URL}}/pyo3/sync/trait.OnceLockExt.html
[`OnceLockExt::get_or_init_py_attached`]: {{#PYO3_DOCS_URL}}/pyo3/sync/trait.OnceLockExt.html#tymethod.get_or_init_py_attached
[`OnceLock`]: https://doc.rust-lang.org/stable/std/sync/struct.OnceLock.html
[`OnceLock::get_or_init`]: https://doc.rust-lang.org/stable/std/sync/struct.OnceLock.html#tymethod.get_or_init
[`Python::allow_threads`]: {{#PYO3_DOCS_URL}}/pyo3/marker/struct.Python.html#method.allow_threads
[`Python::with_gil`]: {{#PYO3_DOCS_URL}}/pyo3/marker/struct.Python.html#method.with_gil
[`Python<'py>`]: {{#PYO3_DOCS_URL}}/pyo3/marker/struct.Python.html
[`threading`]: https://docs.python.org/3/library/threading.html
