# Parallelism

CPython has the infamous [Global Interpreter Lock](https://docs.python.org/3/glossary.html#term-global-interpreter-lock) (GIL), which prevents several threads from executing Python bytecode in parallel. This makes threading in Python a bad fit for [CPU-bound](https://en.wikipedia.org/wiki/CPU-bound) tasks and often forces developers to accept the overhead of multiprocessing. There is an experimental "free-threaded" version of CPython 3.13 that does not have a GIL, see the PyO3 docs on [free-threaded Python](./free-threading.md) for more information about that.

In PyO3 parallelism can be easily achieved in Rust-only code. Let's take a look at our [word-count](https://github.com/PyO3/pyo3/blob/main/examples/word-count/src/lib.rs) example, where we have a `search` function that utilizes the [rayon](https://github.com/rayon-rs/rayon) crate to count words in parallel.
```rust,no_run
# #![allow(dead_code)]
use pyo3::prelude::*;

// These traits let us use `par_lines` and `map`.
use rayon::str::ParallelString;
use rayon::iter::ParallelIterator;

/// Count the occurrences of needle in line, case insensitive
fn count_line(line: &str, needle: &str) -> usize {
    let mut total = 0;
    for word in line.split(' ') {
        if word == needle {
            total += 1;
        }
    }
    total
}

#[pyfunction]
fn search(contents: &str, needle: &str) -> usize {
    contents
        .par_lines()
        .map(|line| count_line(line, needle))
        .sum()
}
```

But let's assume you have a long running Rust function which you would like to execute several times in parallel. For the sake of example let's take a sequential version of the word count:
```rust,no_run
# #![allow(dead_code)]
# fn count_line(line: &str, needle: &str) -> usize {
#     let mut total = 0;
#     for word in line.split(' ') {
#         if word == needle {
#             total += 1;
#         }
#     }
#     total
# }
#
fn search_sequential(contents: &str, needle: &str) -> usize {
    contents.lines().map(|line| count_line(line, needle)).sum()
}
```

To enable parallel execution of this function, the [`Python::detach`] method can be used to temporarily release the GIL, thus allowing other Python threads to run. We then have a function exposed to the Python runtime which calls `search_sequential` inside a closure passed to [`Python::detach`] to enable true parallelism:
```rust,no_run
# #![allow(dead_code)]
# use pyo3::prelude::*;
#
# fn count_line(line: &str, needle: &str) -> usize {
#     let mut total = 0;
#     for word in line.split(' ') {
#         if word == needle {
#             total += 1;
#         }
#     }
#     total
# }
#
# fn search_sequential(contents: &str, needle: &str) -> usize {
#    contents.lines().map(|line| count_line(line, needle)).sum()
# }
#[pyfunction]
fn search_sequential_detached(py: Python<'_>, contents: &str, needle: &str) -> usize {
    py.detach(|| search_sequential(contents, needle))
}
```

Now Python threads can use more than one CPU core, resolving the limitation which usually makes multi-threading in Python only good for IO-bound tasks:
```Python
from concurrent.futures import ThreadPoolExecutor
from word_count import search_sequential_detached

executor = ThreadPoolExecutor(max_workers=2)

future_1 = executor.submit(
    word_count.search_sequential_detached, contents, needle
)
future_2 = executor.submit(
    word_count.search_sequential_detached, contents, needle
)
result_1 = future_1.result()
result_2 = future_2.result()
```

## Benchmark

Let's benchmark the `word-count` example to verify that we really did unlock parallelism with PyO3.

We are using `pytest-benchmark` to benchmark four word count functions:

1. Pure Python version
2. Rust parallel version
3. Rust sequential version
4. Rust sequential version executed twice with two Python threads

The benchmark script can be found [here](https://github.com/PyO3/pyo3/blob/main/examples/word-count/tests/test_word_count.py), and we can run `nox` in the `word-count` folder to benchmark these functions.

While the results of the benchmark of course depend on your machine, the relative results should be similar to this (mid 2020):
```text
-------------------------------------------------------------------------------------------------- benchmark: 4 tests -------------------------------------------------------------------------------------------------
Name (time in ms)                                          Min                Max               Mean            StdDev             Median               IQR            Outliers       OPS            Rounds  Iterations
-----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------
test_word_count_rust_parallel                           1.7315 (1.0)       4.6495 (1.0)       1.9972 (1.0)      0.4299 (1.0)       1.8142 (1.0)      0.2049 (1.0)         40;46  500.6943 (1.0)         375           1
test_word_count_rust_sequential                         7.3348 (4.24)     10.3556 (2.23)      8.0035 (4.01)     0.7785 (1.81)      7.5597 (4.17)     0.8641 (4.22)         26;5  124.9457 (0.25)        121           1
test_word_count_rust_sequential_twice_with_threads      7.9839 (4.61)     10.3065 (2.22)      8.4511 (4.23)     0.4709 (1.10)      8.2457 (4.55)     0.3927 (1.92)        17;17  118.3274 (0.24)        114           1
test_word_count_python_sequential                      27.3985 (15.82)    45.4527 (9.78)     28.9604 (14.50)    4.1449 (9.64)     27.5781 (15.20)    0.4638 (2.26)          3;5   34.5299 (0.07)         35           1
-----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------
```

You can see that the Python threaded version is not much slower than the Rust sequential version, which means compared to an execution on a single CPU core the speed has doubled.

## Sharing Python objects between Rust threads

In the example above we made a Python interface to a low-level rust function,
and then leveraged the python `threading` module to run the low-level function
in parallel. It is also possible to spawn threads in Rust that acquire the GIL
and operate on Python objects. However, care must be taken to avoid writing code
that deadlocks with the GIL in these cases.

* Note: This example is meant to illustrate how to drop and re-acquire the GIL
        to avoid creating deadlocks. Unless the spawned threads subsequently
        release the GIL or you are using the free-threaded build of CPython, you
        will not see any speedups due to multi-threaded parallelism using `rayon`
        to parallelize code that acquires and holds the GIL for the entire
        execution of the spawned thread.

In the example below, we share a `Vec` of User ID objects defined using the
`pyclass` macro and spawn threads to process the collection of data into a `Vec`
of booleans based on a predicate using a `rayon` parallel iterator:

```rust,no_run
use pyo3::prelude::*;

// These traits let us use int_par_iter and map
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};

#[pyclass]
struct UserID {
    id: i64,
}

let allowed_ids: Vec<bool> = Python::attach(|outer_py| {
    let instances: Vec<Py<UserID>> = (0..10).map(|x| Py::new(outer_py, UserID { id: x }).unwrap()).collect();
    outer_py.detach(|| {
        instances.par_iter().map(|instance| {
            Python::attach(|inner_py| {
                instance.borrow(inner_py).id > 5
            })
        }).collect()
    })
});
assert!(allowed_ids.into_iter().filter(|b| *b).count() == 4);
```

It's important to note that there is an `outer_py` Python token as well as
an `inner_py` token. Sharing Python tokens between threads is not allowed
and threads must individually attach to the interpreter to access data wrapped by a Python
object.

It's also important to see that this example uses [`Python::detach`] to
wrap the code that spawns OS threads via `rayon`. If this example didn't use
`detach`, a `rayon` worker thread would block on acquiring the GIL while a
thread that owns the GIL spins forever waiting for the result of the `rayon`
thread. Calling `detach` allows the GIL to be released in the thread
collecting the results from the worker threads. You should always call
`detach` in situations that spawn worker threads, but especially so in
cases where worker threads need to acquire the GIL, to prevent deadlocks.

[`Python::detach`]: {{#PYO3_DOCS_URL}}/pyo3/marker/struct.Python.html#method.detach
