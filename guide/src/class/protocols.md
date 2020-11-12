## Class customizations

Python's object model defines several protocols for different object behavior, like sequence,
mapping or number protocols. PyO3 defines separate traits for each of them. To provide specific
Python object behavior, you need to implement the specific trait for your struct. Important note,
each protocol implementation block has to be annotated with the `#[pyproto]` attribute.

All `#[pyproto]` methods which can be defined below can return `T` instead of `PyResult<T>` if the
method implementation is infallible. In addition, if the return type is `()`, it can be omitted altogether.

### Basic object customization

The [`PyObjectProtocol`] trait provides several basic customizations.

#### Attribute access

To customize object attribute access, define the following methods:

  * `fn __getattr__(&self, name: FromPyObject) -> PyResult<impl IntoPy<PyObject>>`
  * `fn __setattr__(&mut self, name: FromPyObject, value: FromPyObject) -> PyResult<()>`
  * `fn __delattr__(&mut self, name: FromPyObject) -> PyResult<()>`

Each method corresponds to Python's `self.attr`, `self.attr = value` and `del self.attr` code.

#### String Conversions

  * `fn __repr__(&self) -> PyResult<impl ToPyObject<ObjectType=PyString>>`
  * `fn __str__(&self) -> PyResult<impl ToPyObject<ObjectType=PyString>>`

    Possible return types for `__str__` and `__repr__` are `PyResult<String>` or `PyResult<PyString>`.

  * `fn __bytes__(&self) -> PyResult<PyBytes>`

    Provides the conversion to `bytes`.

  * `fn __format__(&self, format_spec: &str) -> PyResult<impl ToPyObject<ObjectType=PyString>>`

    Special method that is used by the `format()` builtin and the `str.format()` method.
    Possible return types are `PyResult<String>` or `PyResult<PyString>`.

#### Comparison operators

  * `fn __richcmp__(&self, other: impl FromPyObject, op: CompareOp) -> PyResult<impl ToPyObject>`

    Overloads Python comparison operations (`==`, `!=`, `<`, `<=`, `>`, and `>=`).
    The `op` argument indicates the comparison operation being performed.
    The return type will normally be `PyResult<bool>`, but any Python object can be returned.
    If `other` is not of the type specified in the signature, the generated code will
    automatically `return NotImplemented`.

  * `fn __hash__(&self) -> PyResult<impl PrimInt>`

    Objects that compare equal must have the same hash value.
    The return type must be `PyResult<T>` where `T` is one of Rust's primitive integer types.

#### Other methods

  * `fn __bool__(&self) -> PyResult<bool>`

    Determines the "truthyness" of the object.

### Emulating numeric types

The [`PyNumberProtocol`] trait allows [emulate numeric types](https://docs.python.org/3/reference/datamodel.html#emulating-numeric-types).

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

These methods are called to implement the binary arithmetic operations
(`+`, `-`, `*`, `@`, `/`, `//`, `%`, `divmod()`, `pow()` and `**`, `<<`, `>>`, `&`, `^`, and `|`).

If `rhs` is not of the type specified in the signature, the generated code
will automatically `return NotImplemented`.  This is not the case for `lhs`
which must match signature or else raise a TypeError.


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

*Note*: Currently implementing the method for a binary arithmetic operations
(e.g, `__add__`) shadows the reflected operation (e.g, `__radd__`).  This is
being addressed in [#844](https://github.com/PyO3/pyo3/issues/844).  to make
these methods


This trait also has support the augmented arithmetic assignments (`+=`, `-=`,
`*=`, `@=`, `/=`, `//=`, `%=`, `**=`, `<<=`, `>>=`, `&=`, `^=`, `|=`):

  * `fn __iadd__(&'p mut self, other: impl FromPyObject) -> PyResult<()>`
  * `fn __isub__(&'p mut self, other: impl FromPyObject) -> PyResult<()>`
  * `fn __imul__(&'p mut self, other: impl FromPyObject) -> PyResult<()>`
  * `fn __imatmul__(&'p mut self, other: impl FromPyObject) -> PyResult<()>`
  * `fn __itruediv__(&'p mut self, other: impl FromPyObject) -> PyResult<()>`
  * `fn __ifloordiv__(&'p mut self, other: impl FromPyObject) -> PyResult<()>`
  * `fn __imod__(&'p mut self, other: impl FromPyObject) -> PyResult<()>`
  * `fn __ipow__(&'p mut self, other: impl FromPyObject) -> PyResult<()>`
  * `fn __ilshift__(&'p mut self, other: impl FromPyObject) -> PyResult<()>`
  * `fn __irshift__(&'p mut self, other: impl FromPyObject) -> PyResult<()>`
  * `fn __iand__(&'p mut self, other: impl FromPyObject) -> PyResult<()>`
  * `fn __ior__(&'p mut self, other: impl FromPyObject) -> PyResult<()>`
  * `fn __ixor__(&'p mut self, other: impl FromPyObject) -> PyResult<()>`

The following methods implement the unary arithmetic operations (`-`, `+`, `abs()` and `~`):

  * `fn __neg__(&'p self) -> PyResult<impl ToPyObject>`
  * `fn __pos__(&'p self) -> PyResult<impl ToPyObject>`
  * `fn __abs__(&'p self) -> PyResult<impl ToPyObject>`
  * `fn __invert__(&'p self) -> PyResult<impl ToPyObject>`

Support for coercions:

  * `fn __complex__(&'p self) -> PyResult<impl ToPyObject>`
  * `fn __int__(&'p self) -> PyResult<impl ToPyObject>`
  * `fn __float__(&'p self) -> PyResult<impl ToPyObject>`

Other:

  * `fn __index__(&'p self) -> PyResult<impl ToPyObject>`
  * `fn __round__(&'p self, ndigits: Option<impl FromPyObject>) -> PyResult<impl ToPyObject>`

### Garbage Collector Integration

If your type owns references to other Python objects, you will need to
integrate with Python's garbage collector so that the GC is aware of
those references.
To do this, implement the [`PyGCProtocol`] trait for your struct.
It includes two methods `__traverse__` and `__clear__`.
These correspond to the slots `tp_traverse` and `tp_clear` in the Python C API.
`__traverse__` must call `visit.call()` for each reference to another Python object.
`__clear__` must clear out any mutable references to other Python objects
(thus breaking reference cycles). Immutable references do not have to be cleared,
as every cycle must contain at least one mutable reference.
Example:
```rust
extern crate pyo3;

use pyo3::prelude::*;
use pyo3::PyTraverseError;
use pyo3::gc::{PyGCProtocol, PyVisit};

#[pyclass]
struct ClassWithGCSupport {
    obj: Option<PyObject>,
}

#[pyproto]
impl PyGCProtocol for ClassWithGCSupport {
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

Special protocol trait implementations have to be annotated with the `#[pyproto]` attribute.

It is also possible to enable GC for custom classes using the `gc` parameter of the `pyclass` attribute.
i.e. `#[pyclass(gc)]`. In that case instances of custom class participate in Python garbage
collection, and it is possible to track them with `gc` module methods. When using the `gc` parameter,
it is *required* to implement the `PyGCProtocol` trait, failure to do so will result in an error
at compile time:

```compile_fail
#[pyclass(gc)]
struct GCTracked {} // Fails because it does not implement PyGCProtocol
```

### Iterator Types

Iterators can be defined using the
[`PyIterProtocol`](https://docs.rs/pyo3/latest/pyo3/class/iter/trait.PyIterProtocol.html) trait.
It includes two methods `__iter__` and `__next__`:
  * `fn __iter__(slf: PyRefMut<Self>) -> PyResult<impl IntoPy<PyObject>>`
  * `fn __next__(slf: PyRefMut<Self>) -> PyResult<Option<impl IntoPy<PyObject>>>`

Returning `None` from `__next__` indicates that that there are no further items.
These two methods can be take either `PyRef<Self>` or `PyRefMut<Self>` as their
first argument, so that mutable borrow can be avoided if needed.

Example:

```rust
use pyo3::prelude::*;
use pyo3::PyIterProtocol;

#[pyclass]
struct MyIterator {
    iter: Box<Iterator<Item = PyObject> + Send>,
}

#[pyproto]
impl PyIterProtocol for MyIterator {
    fn __iter__(slf: PyRef<Self>) -> PyRef<Self> {
        slf
    }
    fn __next__(mut slf: PyRefMut<Self>) -> Option<PyObject> {
        slf.iter.next()
    }
}
```

In many cases you'll have a distinction between the type being iterated over (i.e. the *iterable*) and the iterator it
provides. In this case, you should implement `PyIterProtocol` for both the iterable and the iterator, but the iterable
only needs to support `__iter__()` while the iterator must support both `__iter__()` and `__next__()`. The default
implementations in `PyIterProtocol` will ensure that the objects behave correctly in Python. For example:

```rust
# use pyo3::prelude::*;
# use pyo3::PyIterProtocol;

#[pyclass]
struct Iter {
    inner: std::vec::IntoIter<usize>,
}

#[pyproto]
impl PyIterProtocol for Iter {
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

#[pyproto]
impl PyIterProtocol for Container {
    fn __iter__(slf: PyRef<Self>) -> PyResult<Py<Iter>> {
        let iter = Iter {
            inner: slf.iter.clone().into_iter(),
        };
        Py::new(slf.py(), iter)
    }
}

# let gil = Python::acquire_gil();
# let py = gil.python();
# let inst = pyo3::PyCell::new(
#     py,
#     Container {
#         iter: vec![1, 2, 3, 4],
#     },
# )
# .unwrap();
# pyo3::py_run!(py, inst, "assert list(inst) == [1, 2, 3, 4]");
# pyo3::py_run!(py, inst, "assert list(iter(iter(inst))) == [1, 2, 3, 4]");
```

For more details on Python's iteration protocols, check out [the "Iterator Types" section of the library
documentation](https://docs.python.org/3/library/stdtypes.html#iterator-types).

#### Returning a value from iteration

This guide has so far shown how to use `Option<T>` to implement yielding values during iteration.
In Python a generator can also return a value. To express this in Rust, PyO3 provides the
[`IterNextOutput`](https://docs.rs/pyo3/latest/pyo3/class/iter/enum.IterNextOutput.html) enum to
both `Yield` values and `Return` a final value - see its docs for further details and an example.

[`PyGCProtocol`]: https://docs.rs/pyo3/latest/pyo3/class/gc/trait.PyGCProtocol.html
[`PyGCProtocol`]: https://docs.rs/pyo3/latest/pyo3/class/number/trait.PyNumberProtocol.html
[`PyObjectProtocol`]: https://docs.rs/pyo3/latest/pyo3/class/basic/trait.PyObjectProtocol.html
