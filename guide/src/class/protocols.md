# Magic methods and slots

Python's object model defines several protocols for different object behavior, such as the sequence, mapping, and number protocols. You may be familiar with implementing these protocols in Python classes by "magic" methods, such as `__str__` or `__repr__`. Because of the double-underscores surrounding their name, these are also known as "dunder" methods.

In the Python C-API which PyO3 is implemented upon, many of these magic methods have to be placed into special "slots" on the class type object, as covered in the previous section. There are two ways in which this can be done:

 - [New in PyO3 0.15, recommended in PyO3 0.16] In `#[pymethods]`, if the name of the method is a recognised magic method, PyO3 will place it in the type object automatically.
 - [Deprecated in PyO3 0.16] In special traits combined with the `#[pyproto]` attribute.

(There are also many magic methods which don't have a special slot, such as `__dir__`. These methods can be implemented as normal in `#[pymethods]`.)

If a function name in `#[pymethods]` is a recognised magic method, it will be automatically placed into the correct slot in the Python type object. The function name is taken from the usual rules for naming `#[pymethods]`: the `#[pyo3(name = "...")]` attribute is used if present, otherwise the Rust function name is used.

The magic methods handled by PyO3 are very similar to the standard Python ones on [this page](https://docs.python.org/3/reference/datamodel.html#special-method-names) - in particular they are the the subset which have slots as [defined here](https://docs.python.org/3/c-api/typeobj.html). Some of the slots do not have a magic method in Python, which leads to a few additional magic methods defined only in PyO3:
 - Magic methods for garbage collection
 - Magic methods for the buffer protocol

When PyO3 handles a magic method, a couple of changes apply compared to other `#[pymethods]`:
 - The `#[pyo3(text_signature = "...")]` attribute is not allowed
 - The signature is restricted to match the magic method

The following sections list of all magic methods PyO3 currently handles.  The
given signatures should be interpreted as follows:
 - All methods take a receiver as first argument, shown as `<self>`. It can be
   `&self`, `&mut self` or a `PyCell` reference like `self_: PyRef<Self>` and
   `self_: PyRefMut<Self>`, as described [here](../class.md#inheritance).
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

    Objects that compare equal must have the same hash value.
    <details>
    <summary>Disabling Python's default hash</summary>
    By default, all `#[pyclass]` types have a default hash implementation from Python. Types which should not be hashable can override this by setting `__hash__` to `None`. This is the same mechanism as for a pure-Python class. This is done like so:

    ```rust
    # use pyo3::prelude::*;
    #
    #[pyclass]
    struct NotHashable { }

    #[pymethods]
    impl NotHashable {
        #[classattr]
        const __hash__: Option<PyObject> = None;
    }
    ```
    </details>

  - `__richcmp__(<self>, object, pyo3::basic::CompareOp) -> object`

    Overloads Python comparison operations (`==`, `!=`, `<`, `<=`, `>`, and `>=`).
    The `CompareOp` argument indicates the comparison operation being performed.
    <details>
    <summary>Return type</summary>
    The return type will normally be `PyResult<bool>`, but any Python object can be returned.
    If the `object` is not of the type specified in the signature, the generated code will
    automatically `return NotImplemented`.
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

```rust
use pyo3::prelude::*;

#[pyclass]
struct MyIterator {
    iter: Box<dyn Iterator<Item = PyObject> + Send>,
}

#[pymethods]
impl MyIterator {
    fn __iter__(slf: PyRef<Self>) -> PyRef<Self> {
        slf
    }
    fn __next__(mut slf: PyRefMut<Self>) -> Option<PyObject> {
        slf.iter.next()
    }
}
```

In many cases you'll have a distinction between the type being iterated over
(i.e. the *iterable*) and the iterator it provides. In this case, the iterable
only needs to implement `__iter__()` while the iterator must implement both
`__iter__()` and `__next__()`. For example:

```rust
# use pyo3::prelude::*;

#[pyclass]
struct Iter {
    inner: std::vec::IntoIter<usize>,
}

#[pymethods]
impl Iter {
    fn __iter__(slf: PyRef<Self>) -> PyRef<Self> {
        slf
    }

    fn __next__(mut slf: PyRefMut<Self>) -> Option<usize> {
        slf.inner.next()
    }
}

#[pyclass]
struct Container {
    iter: Vec<usize>,
}

#[pymethods]
impl Container {
    fn __iter__(slf: PyRef<Self>) -> PyResult<Py<Iter>> {
        let iter = Iter {
            inner: slf.iter.clone().into_iter(),
        };
        Py::new(slf.py(), iter)
    }
}

# Python::with_gil(|py| {
#     let container = Container { iter: vec![1, 2, 3, 4] };
#     let inst = pyo3::PyCell::new(py, container).unwrap();
#     pyo3::py_run!(py, inst, "assert list(inst) == [1, 2, 3, 4]");
#     pyo3::py_run!(py, inst, "assert list(iter(iter(inst))) == [1, 2, 3, 4]");
# });
```

For more details on Python's iteration protocols, check out [the "Iterator Types" section of the library
documentation](https://docs.python.org/library/stdtypes.html#iterator-types).

#### Returning a value from iteration

This guide has so far shown how to use `Option<T>` to implement yielding values
during iteration.  In Python a generator can also return a value. To express
this in Rust, PyO3 provides the [`IterNextOutput`] enum to both `Yield` values
and `Return` a final value - see its docs for further details and an example.

### Awaitable objects

  - `__await__(<self>) -> object`
  - `__aiter__(<self>) -> object`
  - `__anext__(<self>) -> Option<object> or IterANextOutput`

### Mapping & Sequence types

  - `__len__(<self>) -> usize`

    Implements the built-in function `len()` for the sequence.

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

    ```rust
    # use pyo3::prelude::*;
    #
    #[pyclass]
    struct NoContains { }

    #[pymethods]
    impl NoContains {
        #[classattr]
        const __contains__: Option<PyObject> = None;
    }
    ```
    </details>

  - `__getitem__(<self>, object) -> object`

    Implements retrieval of the `self[a]` element.

    *Note:* Negative integer indexes are not handled specially.

  - `__setitem__(<self>, object, object) -> ()`

    Implements assignment to the `self[a]` element.
    Should only be implemented if elements can be replaced.

  - `__delitem__(<self>, object) -> ()`

    Implements deletion of the `self[a]` element.
    Should only be implemented if elements can be deleted.

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
  - `__releasebuffer__(<self>, *mut ffi::Py_buffer)` (no return value, not even `PyResult`)

### Garbage Collector Integration

If your type owns references to other Python objects, you will need to integrate
with Python's garbage collector so that the GC is aware of those references.  To
do this, implement the two methods `__traverse__` and `__clear__`.  These
correspond to the slots `tp_traverse` and `tp_clear` in the Python C API.
`__traverse__` must call `visit.call()` for each reference to another Python
object.  `__clear__` must clear out any mutable references to other Python
objects (thus breaking reference cycles). Immutable references do not have to be
cleared, as every cycle must contain at least one mutable reference.

  - `__traverse__(<self>, pyo3::class::gc::PyVisit) -> Result<(), pyo3::class::gc::PyTraverseError>`
  - `__clear__(<self>) -> ()`

Example:

```rust
use pyo3::prelude::*;
use pyo3::PyTraverseError;
use pyo3::gc::PyVisit;

#[pyclass]
struct ClassWithGCSupport {
    obj: Option<PyObject>,
}

#[pymethods]
impl ClassWithGCSupport {
    fn __traverse__(&self, visit: PyVisit) -> Result<(), PyTraverseError> {
        if let Some(obj) = &self.obj {
            visit.call(obj)?
        }
        Ok(())
    }

    fn __clear__(&mut self) {
        // Clear reference, this decrements ref counter.
        self.obj = None;
    }
}
```

[`IterNextOutput`]: {{#PYO3_DOCS_URL}}/pyo3/class/iter/enum.IterNextOutput.html


### `#[pyproto]` traits

PyO3 can use the `#[pyproto]` attribute in combination with special traits to implement the magic methods which need slots. The special traits are listed below. See also the [documentation for the `pyo3::class` module]({{#PYO3_DOCS_URL}}/pyo3/class/index.html).

Due to complexity in the implementation and usage, these traits are deprecated in PyO3 0.16 in favour of the `#[pymethods]` solution.

All `#[pyproto]` methods can return `T` instead of `PyResult<T>` if the method implementation is infallible. In addition, if the return type is `()`, it can be omitted altogether.

#### Basic object customization

The [`PyObjectProtocol`] trait provides several basic customizations.

  * `fn __str__(&self) -> PyResult<impl ToPyObject<ObjectType=PyString>>`
  * `fn __repr__(&self) -> PyResult<impl ToPyObject<ObjectType=PyString>>`
  * `fn __hash__(&self) -> PyResult<impl PrimInt>`
  * `fn __richcmp__(&self, other: impl FromPyObject, op: CompareOp) -> PyResult<impl ToPyObject>`
  * `fn __getattr__(&self, name: impl FromPyObject) -> PyResult<impl IntoPy<PyObject>>`
  * `fn __setattr__(&mut self, name: impl FromPyObject, value: impl FromPyObject) -> PyResult<()>`
  * `fn __delattr__(&mut self, name: impl FromPyObject) -> PyResult<()>`
  * `fn __bool__(&self) -> PyResult<bool>`

#### Emulating numeric types

The [`PyNumberProtocol`] trait can be implemented to emulate [numeric types](https://docs.python.org/3/reference/datamodel.html#emulating-numeric-types).

  * `fn __add__(lhs: impl FromPyObject, rhs: impl FromPyObject) -> PyResult<impl ToPyObject>`
  * `fn __sub__(lhs: impl FromPyObject, rhs: impl FromPyObject) -> PyResult<impl ToPyObject>`
  * `fn __mul__(lhs: impl FromPyObject, rhs: impl FromPyObject) -> PyResult<impl ToPyObject>`
  * `fn __matmul__(lhs: impl FromPyObject, rhs: impl FromPyObject) -> PyResult<impl ToPyObject>`
  * `fn __truediv__(lhs: impl FromPyObject, rhs: impl FromPyObject) -> PyResult<impl ToPyObject>`
  * `fn __floordiv__(lhs: impl FromPyObject, rhs: impl FromPyObject) -> PyResult<impl ToPyObject>`
  * `fn __mod__(lhs: impl FromPyObject, rhs: impl FromPyObject) -> PyResult<impl ToPyObject>`
  * `fn __divmod__(lhs: impl FromPyObject, rhs: impl FromPyObject) -> PyResult<impl ToPyObject>`
  * `fn __pow__(lhs: impl FromPyObject, rhs: impl FromPyObject, modulo: Option<impl FromPyObject>) -> PyResult<impl ToPyObject>`
  * `fn __lshift__(lhs: impl FromPyObject, rhs: impl FromPyObject) -> PyResult<impl ToPyObject>`
  * `fn __rshift__(lhs: impl FromPyObject, rhs: impl FromPyObject) -> PyResult<impl ToPyObject>`
  * `fn __and__(lhs: impl FromPyObject, rhs: impl FromPyObject) -> PyResult<impl ToPyObject>`
  * `fn __or__(lhs: impl FromPyObject, rhs: impl FromPyObject) -> PyResult<impl ToPyObject>`
  * `fn __xor__(lhs: impl FromPyObject, rhs: impl FromPyObject) -> PyResult<impl ToPyObject>`

These methods are called to implement the binary arithmetic operations.

The reflected operations are also available:

  * `fn __radd__(lhs: impl FromPyObject, rhs: impl FromPyObject) -> PyResult<impl ToPyObject>`
  * `fn __rsub__(lhs: impl FromPyObject, rhs: impl FromPyObject) -> PyResult<impl ToPyObject>`
  * `fn __rmul__(lhs: impl FromPyObject, rhs: impl FromPyObject) -> PyResult<impl ToPyObject>`
  * `fn __rmatmul__(lhs: impl FromPyObject, rhs: impl FromPyObject) -> PyResult<impl ToPyObject>`
  * `fn __rtruediv__(lhs: impl FromPyObject, rhs: impl FromPyObject) -> PyResult<impl ToPyObject>`
  * `fn __rfloordiv__(lhs: impl FromPyObject, rhs: impl FromPyObject) -> PyResult<impl ToPyObject>`
  * `fn __rmod__(lhs: impl FromPyObject, rhs: impl FromPyObject) -> PyResult<impl ToPyObject>`
  * `fn __rdivmod__(lhs: impl FromPyObject, rhs: impl FromPyObject) -> PyResult<impl ToPyObject>`
  * `fn __rpow__(lhs: impl FromPyObject, rhs: impl FromPyObject, modulo: Option<impl FromPyObject>) -> PyResult<impl ToPyObject>`
  * `fn __rlshift__(lhs: impl FromPyObject, rhs: impl FromPyObject) -> PyResult<impl ToPyObject>`
  * `fn __rrshift__(lhs: impl FromPyObject, rhs: impl FromPyObject) -> PyResult<impl ToPyObject>`
  * `fn __rand__(lhs: impl FromPyObject, rhs: impl FromPyObject) -> PyResult<impl ToPyObject>`
  * `fn __ror__(lhs: impl FromPyObject, rhs: impl FromPyObject) -> PyResult<impl ToPyObject>`
  * `fn __rxor__(lhs: impl FromPyObject, rhs: impl FromPyObject) -> PyResult<impl ToPyObject>`

The code generated for these methods expect that all arguments match the
signature, or raise a TypeError.

  * `fn __iadd__(&'p mut self, other: impl FromPyObject) -> PyResult<()>`
  * `fn __isub__(&'p mut self, other: impl FromPyObject) -> PyResult<()>`
  * `fn __imul__(&'p mut self, other: impl FromPyObject) -> PyResult<()>`
  * `fn __imatmul__(&'p mut self, other: impl FromPyObject) -> PyResult<()>`
  * `fn __itruediv__(&'p mut self, other: impl FromPyObject) -> PyResult<()>`
  * `fn __ifloordiv__(&'p mut self, other: impl FromPyObject) -> PyResult<()>`
  * `fn __imod__(&'p mut self, other: impl FromPyObject) -> PyResult<()>`
  * `fn __ipow__(&'p mut self, other: impl FromPyObject, modulo: impl FromPyObject) -> PyResult<()>` on Python 3.8^
  * `fn __ipow__(&'p mut self, other: impl FromPyObject) -> PyResult<()>` on Python 3.7 see https://bugs.python.org/issue36379
  * `fn __ilshift__(&'p mut self, other: impl FromPyObject) -> PyResult<()>`
  * `fn __irshift__(&'p mut self, other: impl FromPyObject) -> PyResult<()>`
  * `fn __iand__(&'p mut self, other: impl FromPyObject) -> PyResult<()>`
  * `fn __ior__(&'p mut self, other: impl FromPyObject) -> PyResult<()>`
  * `fn __ixor__(&'p mut self, other: impl FromPyObject) -> PyResult<()>`


The following methods implement the unary arithmetic operations:

  * `fn __neg__(&'p self) -> PyResult<impl ToPyObject>`
  * `fn __pos__(&'p self) -> PyResult<impl ToPyObject>`
  * `fn __abs__(&'p self) -> PyResult<impl ToPyObject>`
  * `fn __invert__(&'p self) -> PyResult<impl ToPyObject>`

Support for coercions:

  * `fn __int__(&'p self) -> PyResult<impl ToPyObject>`
  * `fn __float__(&'p self) -> PyResult<impl ToPyObject>`
  * `fn __index__(&'p self) -> PyResult<impl ToPyObject>`

#### Emulating sequential containers (such as lists or tuples)

The [`PySequenceProtocol`] trait can be implemented to emulate
[sequential container types](https://docs.python.org/3/reference/datamodel.html#emulating-container-types).

For a sequence, the keys are the integers _k_ for which _0 <= k < N_,
where _N_ is the length of the sequence.

  * `fn __len__(&self) -> PyResult<usize>`

    Implements the built-in function `len()` for the sequence.

  * `fn __getitem__(&self, idx: isize) -> PyResult<impl ToPyObject>`

    Implements evaluation of the `self[idx]` element.
    If the `idx` value is outside the set of indexes for the sequence, `IndexError` should be raised.

    *Note:* Negative integer indexes are handled as follows: if `__len__()` is defined,
    it is called and the sequence length is used to compute a positive index,
    which is passed to `__getitem__()`.
    If `__len__()` is not defined, the index is passed as is to the function.

  * `fn __setitem__(&mut self, idx: isize, value: impl FromPyObject) -> PyResult<()>`

    Implements assignment to the `self[idx]` element. Same note as for `__getitem__()`.
    Should only be implemented if sequence elements can be replaced.

  * `fn __delitem__(&mut self, idx: isize) -> PyResult<()>`

    Implements deletion of the `self[idx]` element. Same note as for `__getitem__()`.
    Should only be implemented if sequence elements can be deleted.

  * `fn __contains__(&self, item: impl FromPyObject) -> PyResult<bool>`

    Implements membership test operators.
    Should return true if `item` is in `self`, false otherwise.
    For objects that don’t define `__contains__()`, the membership test simply
    traverses the sequence until it finds a match.

  * `fn __concat__(&self, other: impl FromPyObject) -> PyResult<impl ToPyObject>`

    Concatenates two sequences.
    Used by the `+` operator, after trying the numeric addition via
    the `PyNumberProtocol` trait method.

  * `fn __repeat__(&self, count: isize) -> PyResult<impl ToPyObject>`

    Repeats the sequence `count` times.
    Used by the `*` operator, after trying the numeric multiplication via
    the `PyNumberProtocol` trait method.

  * `fn __inplace_concat__(&mut self, other: impl FromPyObject) -> PyResult<Self>`

    Concatenates two sequences in place. Returns the modified first operand.
    Used by the `+=` operator, after trying the numeric in place addition via
    the `PyNumberProtocol` trait method.

  * `fn __inplace_repeat__(&mut self, count: isize) -> PyResult<Self>`

    Repeats the sequence `count` times in place. Returns the modified first operand.
    Used by the `*=` operator, after trying the numeric in place multiplication via
    the `PyNumberProtocol` trait method.

#### Emulating mapping containers (such as dictionaries)

The [`PyMappingProtocol`] trait allows to emulate
[mapping container types](https://docs.python.org/3/reference/datamodel.html#emulating-container-types).

For a mapping, the keys may be Python objects of arbitrary type.

  * `fn __len__(&self) -> PyResult<usize>`

    Implements the built-in function `len()` for the mapping.

  * `fn __getitem__(&self, key: impl FromPyObject) -> PyResult<impl ToPyObject>`

    Implements evaluation of the `self[key]` element.
    If `key` is of an inappropriate type, `TypeError` may be raised;
    if `key` is missing (not in the container), `KeyError` should be raised.

  * `fn __setitem__(&mut self, key: impl FromPyObject, value: impl FromPyObject) -> PyResult<()>`

    Implements assignment to the `self[key]` element or insertion of a new `key`
    mapping to `value`.
    Should only be implemented if the mapping support changes to the values for keys,
    or if new keys can be added.
    The same exceptions should be raised for improper key values as
    for the `__getitem__()` method.

  * `fn __delitem__(&mut self, key: impl FromPyObject) -> PyResult<()>`

    Implements deletion of the `self[key]` element.
    Should only be implemented if the mapping supports removal of keys.
    The same exceptions should be raised for improper key values as
    for the `__getitem__()` method.

#### Iterator Types

Iterators can be defined using the [`PyIterProtocol`] trait.
It includes two methods `__iter__` and `__next__`:
  * `fn __iter__(slf: PyRefMut<Self>) -> PyResult<impl IntoPy<PyObject>>`
  * `fn __next__(slf: PyRefMut<Self>) -> PyResult<Option<impl IntoPy<PyObject>>>`

These two methods can be take either `PyRef<Self>` or `PyRefMut<Self>` as their
first argument, so that mutable borrow can be avoided if needed.

For details, look at the `#[pymethods]` regarding iterator methods.

#### Garbage Collector Integration

Implement the [`PyGCProtocol`] trait for your struct.
For details, look at the `#[pymethods]` regarding GC methods.

[`PyGCProtocol`]: {{#PYO3_DOCS_URL}}/pyo3/class/gc/trait.PyGCProtocol.html
[`PyMappingProtocol`]: {{#PYO3_DOCS_URL}}/pyo3/class/mapping/trait.PyMappingProtocol.html
[`PyNumberProtocol`]: {{#PYO3_DOCS_URL}}/pyo3/class/number/trait.PyNumberProtocol.html
[`PyObjectProtocol`]: {{#PYO3_DOCS_URL}}/pyo3/class/basic/trait.PyObjectProtocol.html
[`PySequenceProtocol`]: {{#PYO3_DOCS_URL}}/pyo3/class/sequence/trait.PySequenceProtocol.html
[`PyIterProtocol`]: {{#PYO3_DOCS_URL}}/pyo3/class/iter/trait.PyIterProtocol.html
