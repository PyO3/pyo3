# Supporting Free-Threaded CPython

CPython 3.13 introduces an experimental "free-threaded" build of CPython that
does not rely on the [global interpreter
lock](https://docs.python.org/3/glossary.html#term-global-interpreter-lock)
(often referred to as the GIL) for thread safety. As of version 0.23, PyO3 also
has preliminary support for building Rust extensions for the free-threaded
Python build and support for calling into free-threaded Python from Rust.

If you want more background on free-threaded Python in general, see the [what's
new](https://docs.python.org/3/whatsnew/3.13.html#whatsnew313-free-threaded-cpython)
entry in the 3.13 release notes, the [free-threading HOWTO
guide](https://docs.python.org/3/howto/free-threading-extensions.html#freethreading-extensions-howto)
in the CPython docs, the [extension porting
guide](https://py-free-threading.github.io/porting-extensions/) in the
community-maintained Python free-threading guide, and [PEP
703](https://peps.python.org/pep-0703/), which provides the technical background
for the free-threading implementation in CPython.

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

More complicated `#[pyclass]` types may need to deal with thread-safety directly; there is [a dedicated section of the guide](./class/thread-safety.md) to discuss this.

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

```rust,no_run
use pyo3::prelude::*;

/// This module supports free-threaded Python
#[pymodule(gil_used = false)]
fn my_extension(m: &Bound<'_, PyModule>) -> PyResult<()> {
    // add members to the module that you know are thread-safe
    Ok(())
}
```

Or for a module that is set up without using the `pymodule` macro:

```rust,no_run
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

If you would like to use conditional compilation to trigger different code paths
under the free-threaded build, you can use the `Py_GIL_DISABLED` attribute once
you have configured your crate to generate the necessary build configuration
data. See [the guide
section](./building-and-distribution/multiple-python-versions.md) for more
details about supporting multiple different Python versions, including the
free-threaded build.


## Special considerations for the free-threaded build

The free-threaded interpreter does not have a GIL. Many existing extensions
providing mutable data structures relied on the GIL to lock Python objects and
make interior mutability thread-safe.  Historically, PyO3's API was designed
around the same strong assumptions, but is transitioning towards more general
APIs applicable for both builds.

Calling into the CPython C API is only legal when an OS thread is explicitly
attached to the interpreter runtime. In the GIL-enabled build, this happens when
the GIL is acquired. In the free-threaded build there is no GIL, but the same C
macros that release or acquire the GIL in the GIL-enabled build instead ask the
interpreter to attach the thread to the Python runtime, and there can be many
threads simultaneously attached. See [PEP
703](https://peps.python.org/pep-0703/#thread-states) for more background about
how threads can be attached and detached from the interpreter runtime, in a
manner analogous to releasing and acquiring the GIL in the GIL-enabled build.

In the GIL-enabled build, PyO3 uses the [`Python<'py>`] type and the `'py`
lifetime to signify that the global interpreter lock is held. In the
freethreaded build, holding a `'py` lifetime means only that the thread is
currently attached to the Python interpreter -- other threads can be
simultaneously interacting with the interpreter.

### Attaching to the runtime

You still need to obtain a `'py` lifetime to interact with Python
objects or call into the CPython C API. If you are not yet attached to the
Python runtime, you can register a thread using the [`Python::attach`]
function. Threads created via the Python [`threading`] module do not need to
do this, and pyo3 will handle setting up the [`Python<'py>`] token when CPython
calls into your extension.

### Detaching to avoid hangs and deadlocks

The free-threaded build triggers global synchronization events in the following
situations:

* During garbage collection in order to get a globally consistent view of
  reference counts and references between objects
* In Python 3.13, when the first background thread is started in
  order to mark certain objects as immortal
* When either `sys.settrace` or `sys.setprofile` are called in order to
  instrument running code objects and threads
* During a call to `os.fork()`, to ensure a process-wide consistent state.

This is a non-exhaustive list and there may be other situations in future Python
versions that can trigger global synchronization events.

This means that you should detach from the interpreter runtime using
[`Python::detach`] in exactly the same situations as you should detach
from the runtime in the GIL-enabled build: when doing long-running tasks that do
not require the CPython runtime or when doing any task that needs to re-attach
to the runtime (see the [guide
section](parallelism.md#sharing-python-objects-between-rust-threads) that
covers this). In the former case, you would observe a hang on threads that are
waiting on the long-running task to complete, and in the latter case you would
see a deadlock while a thread tries to attach after the runtime triggers a
global synchronization event, but the spawning thread prevents the
synchronization event from completing.

### Exceptions and panics for multithreaded access of mutable `pyclass` instances

Data attached to `pyclass` instances is protected from concurrent access by a
`RefCell`-like pattern of runtime borrow checking. Like a `RefCell`, PyO3 will
raise exceptions (or in some cases panic) to enforce exclusive access for
mutable borrows. It was always possible to generate panics like this in PyO3 in
code that releases the GIL with [`Python::detach`] or calling a python
method accepting `&self` from a `&mut self` (see [the docs on interior
mutability](./class.md#bound-and-interior-mutability),) but now in free-threaded
Python there are more opportunities to trigger these panics from Python because
there is no GIL to lock concurrent access to mutably borrowed data from Python.

The most straightforward way to trigger this problem is to use the Python
[`threading`] module to simultaneously call a rust function that mutably borrows a
[`pyclass`]({{#PYO3_DOCS_URL}}/pyo3/attr.pyclass.html) in multiple threads. For
example, consider the following implementation:

```rust,no_run
# use pyo3::prelude::*;
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

We may allow user-selectable semantics for mutable pyclass definitions in a
future version of PyO3, allowing some form of opt-in locking to emulate the GIL
if that is needed. For now you should explicitly add locking, possibly using
conditional compilation or using the critical section API, to avoid creating
deadlocks with the GIL.

### Cannot build extensions using the limited API

The free-threaded build uses a completely new ABI and there is not yet an
equivalent to the limited API for the free-threaded ABI. That means if your
crate depends on PyO3 using the `abi3` feature or an an `abi3-pyxx` feature,
PyO3 will print a warning and ignore that setting when building extensions using
the free-threaded interpreter.

This means that if your package makes use of the ABI forward compatibility
provided by the limited API to upload only one wheel for each release of your
package, you will need to update your release procedure to also upload a
version-specific free-threaded wheel.

See [the guide section](./building-and-distribution/multiple-python-versions.md)
for more details about supporting multiple different Python versions, including
the free-threaded build.

### Thread-safe single initialization

To initialize data exactly once, use the [`PyOnceLock`] type, which is a close equivalent
to [`std::sync::OnceLock`][`OnceLock`] that also helps avoid deadlocks by detaching from
the Python interpreter when threads are blocking waiting for another thread to
complete intialization. If already using [`OnceLock`] and it is impractical
to replace with a [`PyOnceLock`], there is the [`OnceLockExt`] extension trait
which adds [`OnceLockExt::get_or_init_py_attached`] to detach from the interpreter
when blocking in the same fashion as [`PyOnceLock`]. Here is an example using
[`PyOnceLock`] to single-initialize a runtime cache holding a `Py<PyDict>`:

```rust
# use pyo3::prelude::*;
use pyo3::sync::PyOnceLock;
use pyo3::types::PyDict;

let cache: PyOnceLock<Py<PyDict>> = PyOnceLock::new();

Python::attach(|py| {
    // guaranteed to be called once and only once
    cache.get_or_init(py, || PyDict::new(py).unbind())
});
```

In cases where a function must run exactly once, you can bring
the [`OnceExt`] trait into scope. The [`OnceExt`] trait adds
[`OnceExt::call_once_py_attached`] and [`OnceExt::call_once_force_py_attached`]
functions to the api of `std::sync::Once`, enabling use of [`Once`] in contexts
where the thread is attached to the Python interpreter. These functions are analogous to
[`Once::call_once`], [`Once::call_once_force`] except they accept a [`Python<'py>`]
token in addition to an `FnOnce`. All of these functions detach from the
interpreter before blocking and re-attach before executing the function,
avoiding deadlocks that are possible without using the PyO3
extension traits. Here the same example as above built using a [`Once`] instead of a
[`PyOnceLock`]:

```rust
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

Python::attach(|py| {
    // guaranteed to be called once and only once
    cache.once.call_once_py_attached(py, || {
        cache.cache = Some(PyDict::new(py).unbind());
    });
});
```

### `GILProtected` is not exposed

[`GILProtected`] is a (deprecated) PyO3 type that allows mutable access to static data by
leveraging the GIL to lock concurrent access from other threads. In
free-threaded Python there is no GIL, so you will need to replace this type with
some other form of locking. In many cases, a type from
[`std::sync::atomic`](https://doc.rust-lang.org/std/sync/atomic/) or a
[`std::sync::Mutex`](https://doc.rust-lang.org/std/sync/struct.Mutex.html) will
be sufficient.

Before:

```rust
# #![allow(deprecated)]
# fn main() {
# #[cfg(not(Py_GIL_DISABLED))] {
# use pyo3::prelude::*;
use pyo3::sync::GILProtected;
use pyo3::types::{PyDict, PyNone};
use std::cell::RefCell;

static OBJECTS: GILProtected<RefCell<Vec<Py<PyDict>>>> =
    GILProtected::new(RefCell::new(Vec::new()));

Python::attach(|py| {
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

Python::attach(|py| {
    // stand-in for something that executes arbitrary Python code
    let d = PyDict::new(py);
    d.set_item(PyNone::get(py), PyNone::get(py)).unwrap();
    // as with any `Mutex` usage, lock the mutex for as little time as possible
    // in this case, we do it just while pushing into the `Vec`
    OBJECTS.lock().unwrap().push(d.unbind());
});
# }
```

If you are executing arbitrary Python code while holding the lock, then you
should import the [`MutexExt`] trait and use the `lock_py_attached` method
instead of `lock`. This ensures that global synchronization events started by
the Python runtime can proceed, avoiding possible deadlocks with the
interpreter.

[`GILProtected`]: https://docs.rs/pyo3/0.22/pyo3/sync/struct.GILProtected.html
[`MutexExt`]: {{#PYO3_DOCS_URL}}/pyo3/sync/trait.MutexExt.html
[`Once`]: https://doc.rust-lang.org/stable/std/sync/struct.Once.html
[`Once::call_once`]: https://doc.rust-lang.org/stable/std/sync/struct.Once.html#method.call_once
[`Once::call_once_force`]: https://doc.rust-lang.org/stable/std/sync/struct.Once.html#method.call_once_force
[`OnceExt`]: {{#PYO3_DOCS_URL}}/pyo3/sync/trait.OnceExt.html
[`OnceExt::call_once_py_attached`]: {{#PYO3_DOCS_URL}}/pyo3/sync/trait.OnceExt.html#tymethod.call_once_py_attached
[`OnceExt::call_once_force_py_attached`]: {{#PYO3_DOCS_URL}}/pyo3/sync/trait.OnceExt.html#tymethod.call_once_force_py_attached
[`OnceLockExt`]: {{#PYO3_DOCS_URL}}/pyo3/sync/trait.OnceLockExt.html
[`OnceLockExt::get_or_init_py_attached`]: {{#PYO3_DOCS_URL}}/pyo3/sync/trait.OnceLockExt.html#tymethod.get_or_init_py_attached
[`OnceLock`]: https://doc.rust-lang.org/stable/std/sync/struct.OnceLock.html
[`OnceLock::get_or_init`]: https://doc.rust-lang.org/stable/std/sync/struct.OnceLock.html#method.get_or_init
[`Python::detach`]: {{#PYO3_DOCS_URL}}/pyo3/marker/struct.Python.html#method.detach
[`Python::attach`]: {{#PYO3_DOCS_URL}}/pyo3/marker/struct.Python.html#method.attach
[`Python<'py>`]: {{#PYO3_DOCS_URL}}/pyo3/marker/struct.Python.html
[`PyOnceLock`]: {{#PYO3_DOCS_URL}}/pyo3/sync/struct.PyOnceLock.html
[`threading`]: https://docs.python.org/3/library/threading.html
