# Class customizations

Python's object model defines several protocols for different object behavior, such as the sequence, mapping, and number protocols. Python classes support these protocols by implementing "magic" methods, such as `__str__` or `__repr__`. Because of the double-underscores surrounding their name, these are also known as "dunder" methods.

PyO3 makes it possible for every magic method to be implemented in `#[pymethods]` just as they would be done in a regular Python class, with a few notable differences:
- `__new__` and `__init__` are replaced by the [`#[new]` attribute](../class.md#constructor).
- `__del__` is not yet supported, but may be in the future.
- `__buffer__` and `__release_buffer__` are currently not supported and instead PyO3 supports [`__getbuffer__` and `__releasebuffer__`](#buffer-objects) methods (these predate [PEP 688](https://peps.python.org/pep-0688/#python-level-buffer-protocol)), again this may change in the future.
- PyO3 adds [`__traverse__` and `__clear__`](#garbage-collector-integration) methods for controlling garbage collection.
- The Python C-API which PyO3 is implemented upon requires many magic methods to have a specific function signature in C and be placed into special "slots" on the class type object. This limits the allowed argument and return types for these methods. They are listed in detail in the section below.

If a magic method is not on the list above (for example `__init_subclass__`), then it should just work in PyO3. If this is not the case, please file a bug report.

## Magic Methods handled by PyO3

If a function name in `#[pymethods]` is a magic method which is known to need special handling, it will be automatically placed into the correct slot in the Python type object. The function name is taken from the usual rules for naming `#[pymethods]`: the `#[pyo3(name = "...")]` attribute is used if present, otherwise the Rust function name is used.

The magic methods handled by PyO3 are very similar to the standard Python ones on [this page](https://docs.python.org/3/reference/datamodel.html#special-method-names) - in particular they are the subset which have slots as [defined here](https://docs.python.org/3/c-api/typeobj.html).

When PyO3 handles a magic method, a couple of changes apply compared to other `#[pymethods]`:
 - The Rust function signature is restricted to match the magic method.
 - The `#[pyo3(signature = (...)]` and `#[pyo3(text_signature = "...")]` attributes are not allowed.

The following sections list all magic methods for which PyO3 implements the necessary special handling.  The
given signatures should be interpreted as follows:
 - All methods take a receiver as first argument, shown as `<self>`. It can be
   `&self`, `&mut self` or a `Bound` reference like `self_: PyRef<'_, Self>` and
   `self_: PyRefMut<'_, Self>`, as described [here](../class.md#inheritance).
 - An optional `Python<'py>` argument is always allowed as the first argument.
 - Return values can be optionally wrapped in `PyResult`.
 - `object` means that any type is allowed that can be extracted from a Python
   object (if argument) or converted to a Python object (if return value).
 - Other types must match what's given, e.g. `pyo3::basic::CompareOp` for
   `__richcmp__`'s second argument.
 - For the comparison and arithmetic methods, extraction errors are not
   propagated as exceptions, but lead to a return of `NotImplemented`.
 - For some magic methods, the return values are not restricted by PyO3, but
   checked by the Python interpreter. For example, `__str__` needs to return a
   string object.  This is indicated by `object (Python type)`.

### Basic object customization

  - `__str__(<self>) -> object (str)`
  - `__repr__(<self>) -> object (str)`

  - `__hash__(<self>) -> isize`

    Objects that compare equal must have the same hash value. Any type up to 64 bits may be returned instead of `isize`, PyO3 will convert to an isize automatically (wrapping unsigned types like `u64` and `usize`).
    <details>
    <summary>Disabling Python's default hash</summary>
    By default, all `#[pyclass]` types have a default hash implementation from Python. Types which should not be hashable can override this by setting `__hash__` to `None`. This is the same mechanism as for a pure-Python class. This is done like so:

    ```rust,no_run
    # use pyo3::prelude::*;
    #
    #[pyclass]
    struct NotHashable {}

    #[pymethods]
    impl NotHashable {
        #[classattr]
        const __hash__: Option<Py<PyAny>> = None;
    }
    ```
    </details>

  - `__lt__(<self>, object) -> object`
  - `__le__(<self>, object) -> object`
  - `__eq__(<self>, object) -> object`
  - `__ne__(<self>, object) -> object`
  - `__gt__(<self>, object) -> object`
  - `__ge__(<self>, object) -> object`

    The implementations of Python's "rich comparison" operators `<`, `<=`, `==`, `!=`, `>` and `>=` respectively.

    _Note that implementing any of these methods will cause Python not to generate a default `__hash__` implementation, so consider also implementing `__hash__`._
    <details>
    <summary>Return type</summary>
    The return type will normally be `bool` or `PyResult<bool>`, however any Python object can be returned.
    </details>

  - `__richcmp__(<self>, object, pyo3::basic::CompareOp) -> object`

    Implements Python comparison operations (`==`, `!=`, `<`, `<=`, `>`, and `>=`) in a single method.
    The `CompareOp` argument indicates the comparison operation being performed. You can use
    [`CompareOp::matches`] to adapt a Rust `std::cmp::Ordering` result to the requested comparison.

    _This method cannot be implemented in combination with any of `__lt__`, `__le__`, `__eq__`, `__ne__`, `__gt__`, or `__ge__`._

    _Note that implementing `__richcmp__` will cause Python not to generate a default `__hash__` implementation, so consider implementing `__hash__` when implementing `__richcmp__`._
    <details>
    <summary>Return type</summary>
    The return type will normally be `PyResult<bool>`, but any Python object can be returned.

    If you want to leave some operations unimplemented, you can return `py.NotImplemented()`
    for some of the operations:

    ```rust,no_run
    use pyo3::class::basic::CompareOp;
    use pyo3::types::PyNotImplemented;

    # use pyo3::prelude::*;
    # use pyo3::BoundObject;
    #
    # #[pyclass]
    # struct Number(i32);
    #
    #[pymethods]
    impl Number {
        fn __richcmp__<'py>(&self, other: &Self, op: CompareOp, py: Python<'py>) -> PyResult<Borrowed<'py, 'py, PyAny>> {
            match op {
                CompareOp::Eq => Ok((self.0 == other.0).into_pyobject(py)?.into_any()),
                CompareOp::Ne => Ok((self.0 != other.0).into_pyobject(py)?.into_any()),
                _ => Ok(PyNotImplemented::get(py).into_any()),
            }
        }
    }
    ```

    If the second argument `object` is not of the type specified in the
    signature, the generated code will automatically `return NotImplemented`.
    </details>

  - `__getattr__(<self>, object) -> object`
  - `__getattribute__(<self>, object) -> object`
    <details>
    <summary>Differences between `__getattr__` and `__getattribute__`</summary>
    As in Python, `__getattr__` is only called if the attribute is not found
    by normal attribute lookup.  `__getattribute__`, on the other hand, is
    called for *every* attribute access.  If it wants to access existing
    attributes on `self`, it needs to be very careful not to introduce
    infinite recursion, and use `baseclass.__getattribute__()`.
    </details>

  - `__setattr__(<self>, value: object) -> ()`
  - `__delattr__(<self>, object) -> ()`

    Overrides attribute access.

  - `__bool__(<self>) -> bool`

    Determines the "truthyness" of an object.

  - `__call__(<self>, ...) -> object` - here, any argument list can be defined
    as for normal `pymethods`

### Iterable objects

Iterators can be defined using these methods:

  - `__iter__(<self>) -> object`
  - `__next__(<self>) -> Option<object> or IterNextOutput` ([see details](#returning-a-value-from-iteration))

Returning `None` from `__next__` indicates that that there are no further items.

Example:

```rust,no_run
use pyo3::prelude::*;

use std::sync::Mutex;

#[pyclass]
struct MyIterator {
    iter: Mutex<Box<dyn Iterator<Item = Py<PyAny>> + Send>>,
}

#[pymethods]
impl MyIterator {
    fn __iter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        slf
    }
    fn __next__(slf: PyRefMut<'_, Self>) -> Option<Py<PyAny>> {
        slf.iter.lock().unwrap().next()
    }
}
```

In many cases you'll have a distinction between the type being iterated over
(i.e. the *iterable*) and the iterator it provides. In this case, the iterable
only needs to implement `__iter__()` while the iterator must implement both
`__iter__()` and `__next__()`. For example:

```rust,no_run
# use pyo3::prelude::*;

#[pyclass]
struct Iter {
    inner: std::vec::IntoIter<usize>,
}

#[pymethods]
impl Iter {
    fn __iter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        slf
    }

    fn __next__(mut slf: PyRefMut<'_, Self>) -> Option<usize> {
        slf.inner.next()
    }
}

#[pyclass]
struct Container {
    iter: Vec<usize>,
}

#[pymethods]
impl Container {
    fn __iter__(slf: PyRef<'_, Self>) -> PyResult<Py<Iter>> {
        let iter = Iter {
            inner: slf.iter.clone().into_iter(),
        };
        Py::new(slf.py(), iter)
    }
}

# Python::attach(|py| {
#     let container = Container { iter: vec![1, 2, 3, 4] };
#     let inst = pyo3::Py::new(py, container).unwrap();
#     pyo3::py_run!(py, inst, "assert list(inst) == [1, 2, 3, 4]");
#     pyo3::py_run!(py, inst, "assert list(iter(iter(inst))) == [1, 2, 3, 4]");
# });
```

For more details on Python's iteration protocols, check out [the "Iterator Types" section of the library
documentation](https://docs.python.org/library/stdtypes.html#iterator-types).

#### Returning a value from iteration

This guide has so far shown how to use `Option<T>` to implement yielding values
during iteration.  In Python a generator can also return a value. This is done by
raising a `StopIteration` exception. To express this in Rust, return `PyResult::Err`
with a `PyStopIteration` as the error.

### Awaitable objects

  - `__await__(<self>) -> object`
  - `__aiter__(<self>) -> object`
  - `__anext__(<self>) -> Option<object>`

### Mapping & Sequence types

The magic methods in this section can be used to implement Python container types. They are two main categories of container in Python: "mappings" such as `dict`, with arbitrary keys, and "sequences" such as `list` and `tuple`, with integer keys.

The Python C-API which PyO3 is built upon has separate "slots" for sequences and mappings. When writing a `class` in pure Python, there is no such distinction in the implementation - a `__getitem__` implementation will fill the slots for both the mapping and sequence forms, for example.

By default PyO3 reproduces the Python behaviour of filling both mapping and sequence slots. This makes sense for the "simple" case which matches Python, and also for sequences, where the mapping slot is used anyway to implement slice indexing.

Mapping types usually will not want the sequence slots filled. Having them filled will lead to outcomes which may be unwanted, such as:
- The mapping type will successfully cast to [`PySequence`]. This may lead to consumers of the type handling it incorrectly.
- Python provides a default implementation of `__iter__` for sequences, which calls `__getitem__` with consecutive positive integers starting from 0 until an `IndexError` is returned. Unless the mapping only contains consecutive positive integer keys, this `__iter__` implementation will likely not be the intended behavior.

Use the `#[pyclass(mapping)]` annotation to instruct PyO3 to only fill the mapping slots, leaving the sequence ones empty. This will apply to `__getitem__`, `__setitem__`, and `__delitem__`.

Use the `#[pyclass(sequence)]` annotation to instruct PyO3 to fill the `sq_length` slot instead of the `mp_length` slot for `__len__`. This will help libraries such as `numpy` recognise the class as a sequence, however will also cause CPython to automatically add the sequence length to any negative indices before passing them to `__getitem__`. (`__getitem__`, `__setitem__` and `__delitem__` mapping slots are still used for sequences, for slice operations.)

  - `__len__(<self>) -> usize`

    Implements the built-in function `len()`.

  - `__contains__(<self>, object) -> bool`

    Implements membership test operators.
    Should return true if `item` is in `self`, false otherwise.
    For objects that don’t define `__contains__()`, the membership test simply
    traverses the sequence until it finds a match.

    <details>
    <summary>Disabling Python's default contains</summary>

    By default, all `#[pyclass]` types with an `__iter__` method support a
    default implementation of the `in` operator. Types which do not want this
    can override this by setting `__contains__` to `None`. This is the same
    mechanism as for a pure-Python class. This is done like so:

    ```rust,no_run
    # use pyo3::prelude::*;
    #
    #[pyclass]
    struct NoContains {}

    #[pymethods]
    impl NoContains {
        #[classattr]
        const __contains__: Option<Py<PyAny>> = None;
    }
    ```
    </details>

  - `__getitem__(<self>, object) -> object`

    Implements retrieval of the `self[a]` element.

    *Note:* Negative integer indexes are not handled specially by PyO3.
    However, for classes with `#[pyclass(sequence)]`, when a negative index is
    accessed via `PySequence::get_item`, the underlying C API already adjusts
    the index to be positive.

  - `__setitem__(<self>, object, object) -> ()`

    Implements assignment to the `self[a]` element.
    Should only be implemented if elements can be replaced.

    Same behavior regarding negative indices as for `__getitem__`.

  - `__delitem__(<self>, object) -> ()`

    Implements deletion of the `self[a]` element.
    Should only be implemented if elements can be deleted.

    Same behavior regarding negative indices as for `__getitem__`.

  * `fn __concat__(&self, other: impl FromPyObject) -> PyResult<impl ToPyObject>`

    Concatenates two sequences.
    Used by the `+` operator, after trying the numeric addition via
    the `__add__` and `__radd__` methods.

  * `fn __repeat__(&self, count: isize) -> PyResult<impl ToPyObject>`

    Repeats the sequence `count` times.
    Used by the `*` operator, after trying the numeric multiplication via
    the `__mul__` and `__rmul__` methods.

  * `fn __inplace_concat__(&self, other: impl FromPyObject) -> PyResult<impl ToPyObject>`

    Concatenates two sequences.
    Used by the `+=` operator, after trying the numeric addition via
    the `__iadd__` method.

  * `fn __inplace_repeat__(&self, count: isize) -> PyResult<impl ToPyObject>`

    Concatenates two sequences.
    Used by the `*=` operator, after trying the numeric multiplication via
    the `__imul__` method.

### Descriptors

  - `__get__(<self>, object, object) -> object`
  - `__set__(<self>, object, object) -> ()`
  - `__delete__(<self>, object) -> ()`

### Numeric types

Binary arithmetic operations (`+`, `-`, `*`, `@`, `/`, `//`, `%`, `divmod()`,
`pow()` and `**`, `<<`, `>>`, `&`, `^`, and `|`) and their reflected versions:

(If the `object` is not of the type specified in the signature, the generated code
will automatically `return NotImplemented`.)

  - `__add__(<self>, object) -> object`
  - `__radd__(<self>, object) -> object`
  - `__sub__(<self>, object) -> object`
  - `__rsub__(<self>, object) -> object`
  - `__mul__(<self>, object) -> object`
  - `__rmul__(<self>, object) -> object`
  - `__matmul__(<self>, object) -> object`
  - `__rmatmul__(<self>, object) -> object`
  - `__floordiv__(<self>, object) -> object`
  - `__rfloordiv__(<self>, object) -> object`
  - `__truediv__(<self>, object) -> object`
  - `__rtruediv__(<self>, object) -> object`
  - `__divmod__(<self>, object) -> object`
  - `__rdivmod__(<self>, object) -> object`
  - `__mod__(<self>, object) -> object`
  - `__rmod__(<self>, object) -> object`
  - `__lshift__(<self>, object) -> object`
  - `__rlshift__(<self>, object) -> object`
  - `__rshift__(<self>, object) -> object`
  - `__rrshift__(<self>, object) -> object`
  - `__and__(<self>, object) -> object`
  - `__rand__(<self>, object) -> object`
  - `__xor__(<self>, object) -> object`
  - `__rxor__(<self>, object) -> object`
  - `__or__(<self>, object) -> object`
  - `__ror__(<self>, object) -> object`
  - `__pow__(<self>, object, object) -> object`
  - `__rpow__(<self>, object, object) -> object`

In-place assignment operations (`+=`, `-=`, `*=`, `@=`, `/=`, `//=`, `%=`,
`**=`, `<<=`, `>>=`, `&=`, `^=`, `|=`):

  - `__iadd__(<self>, object) -> ()`
  - `__isub__(<self>, object) -> ()`
  - `__imul__(<self>, object) -> ()`
  - `__imatmul__(<self>, object) -> ()`
  - `__itruediv__(<self>, object) -> ()`
  - `__ifloordiv__(<self>, object) -> ()`
  - `__imod__(<self>, object) -> ()`
  - `__ipow__(<self>, object, object) -> ()`
  - `__ilshift__(<self>, object) -> ()`
  - `__irshift__(<self>, object) -> ()`
  - `__iand__(<self>, object) -> ()`
  - `__ixor__(<self>, object) -> ()`
  - `__ior__(<self>, object) -> ()`

Unary operations (`-`, `+`, `abs()` and `~`):

  - `__pos__(<self>) -> object`
  - `__neg__(<self>) -> object`
  - `__abs__(<self>) -> object`
  - `__invert__(<self>) -> object`

Coercions:

  - `__index__(<self>) -> object (int)`
  - `__int__(<self>) -> object (int)`
  - `__float__(<self>) -> object (float)`

### Buffer objects

  - `__getbuffer__(<self>, *mut ffi::Py_buffer, flags) -> ()`
  - `__releasebuffer__(<self>, *mut ffi::Py_buffer) -> ()`
    Errors returned from `__releasebuffer__` will be sent to `sys.unraiseablehook`. It is strongly advised to never return an error from `__releasebuffer__`, and if it really is necessary, to make best effort to perform any required freeing operations before returning. `__releasebuffer__` will not be called a second time; anything not freed will be leaked.

### Garbage Collector Integration

If your type owns references to other Python objects, you will need to integrate
with Python's garbage collector so that the GC is aware of those references.  To
do this, implement the two methods `__traverse__` and `__clear__`.  These
correspond to the slots `tp_traverse` and `tp_clear` in the Python C API.
`__traverse__` must call `visit.call()` for each reference to another Python
object.  `__clear__` must clear out any mutable references to other Python
objects (thus breaking reference cycles). Immutable references do not have to be
cleared, as every cycle must contain at least one mutable reference.

  - `__traverse__(<self>, pyo3::class::gc::PyVisit<'_>) -> Result<(), pyo3::class::gc::PyTraverseError>`
  - `__clear__(<self>) -> ()`

> Note: `__traverse__` does not work with [`#[pyo3(warn(...))]`](../function.md#warn).

Example:

```rust,no_run
use pyo3::prelude::*;
use pyo3::PyTraverseError;
use pyo3::gc::PyVisit;

#[pyclass]
struct ClassWithGCSupport {
    obj: Option<Py<PyAny>>,
}

#[pymethods]
impl ClassWithGCSupport {
    fn __traverse__(&self, visit: PyVisit<'_>) -> Result<(), PyTraverseError> {
        visit.call(&self.obj)?;
        Ok(())
    }

    fn __clear__(&mut self) {
        // Clear reference, this decrements ref counter.
        self.obj = None;
    }
}
```

Usually, an implementation of `__traverse__` should do nothing but calls to `visit.call`.
Most importantly, safe access to the interpreter is prohibited inside implementations of `__traverse__`,
i.e. `Python::attach` will panic.

> Note: these methods are part of the C API, PyPy does not necessarily honor them. If you are building for PyPy you should measure memory consumption to make sure you do not have runaway memory growth. See [this issue on the PyPy bug tracker](https://github.com/pypy/pypy/issues/3848).

[`PySequence`]: {{#PYO3_DOCS_URL}}/pyo3/types/struct.PySequence.html
[`CompareOp::matches`]: {{#PYO3_DOCS_URL}}/pyo3/pyclass/enum.CompareOp.html#method.matches
