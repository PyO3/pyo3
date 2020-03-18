# Appendix B: Migrating from older PyO3 versions
This guide can help you upgrade code through breaking changes from one PyO3 version to the next. For a detailed list of all changes, see [CHANGELOG.md](https://github.com/PyO3/pyo3/blob/master/CHANGELOG.md)
## from 0.8.* to 0.9

### `#[new]` interface
[`PyRawObject`](https://docs.rs/pyo3/0.8.5/pyo3/type_object/struct.PyRawObject.html)
is now removed and our syntax for constructors has changed.

Before:
```rust,compile_fail
#[pyclass]
struct MyClass {}

#[pymethods]
impl MyClass {
   #[new]
   fn new(obj: &PyRawObject) {
       obj.init(MyClass { })
   }
}
```

After:
```rust
# use pyo3::prelude::*;
#[pyclass]
struct MyClass {}

#[pymethods]
impl MyClass {
   #[new]
   fn new() -> Self {
       MyClass {}
   }
}
```

Basically you can return `Self` or `Result<Self>` directly.
For more, see [the constructor section](https://pyo3.rs/master/class.html#constructor) of this guide.

### PyCell
PyO3 0.9 introduces [`PyCell`], which is a [`RefCell`]-like object wrapper
for ensuring Rust's rules regarding aliasing of references are upheld.
For more detail, see the
[Rust Book's section on Rust's rules of references](https://doc.rust-lang.org/book/ch04-02-references-and-borrowing.html#the-rules-of-references)

For `#[pymethods]` or `#[pyfunction]`s, your existing code should continue to work without any change.
Python exceptions will automatically be raised when your functions are used in a way which breaks Rust's
rules of references.

Here is an example.
```rust
# use pyo3::prelude::*;
#[pyclass]
struct Names {
    names: Vec<String>
}

#[pymethods]
impl Names {
    #[new]
    fn new() -> Self {
        Names { names: vec![] }
    }
    fn merge(&mut self, other: &mut Names) {
        self.names.append(&mut other.names)
    }
}
# let gil = Python::acquire_gil();
# let py = gil.python();
# let names = PyCell::new(py, Names::new()).unwrap();
# let borrow_mut_err = py.get_type::<pyo3::pycell::PyBorrowMutError>();
# pyo3::py_run!(py, names borrow_mut_err, r"
# try:
#    names.merge(names)
#    assert False, 'Unreachable'
# except RuntimeError as e:
#    isinstance(e, borrow_mut_err)
# ");
```
`Names` has a `merge` method, which takes `&mut self` and another argument of type `&mut Self`.
Given this `#[pyclass]`, calling `names.merge(names)` in Python raises
a [`PyBorrowMutError`] exception, since it requires two mutable borrows of `names`.

However, for `#[pyproto]` and some functions, you need to manually fix the code.

#### Object creation
In 0.8 object creation was done with `PyRef::new` and `PyRefMut::new`.
In 0.9 these have both been removed.
To upgrade code, please use
[`PyCell::new`](https://pyo3.rs/master/doc/pyo3/pycell/struct.PyCell.html#method.new) instead.
If you need [`PyRef`] or [`PyRefMut`], just call `.borrow()` or `.borrow_mut()`
on the newly-created `PyCell`.

Before:
```rust,compile_fail
# use pyo3::prelude::*;
# #[pyclass]
# struct MyClass {}
let gil = Python::acquire_gil();
let py = gil.python();
let obj_ref = PyRef::new(py, MyClass {}).unwrap();
```

After:
```rust
# use pyo3::prelude::*;
# #[pyclass]
# struct MyClass {}
let gil = Python::acquire_gil();
let py = gil.python();
let obj = PyCell::new(py, MyClass {}).unwrap();
let obj_ref = obj.borrow();
```

#### Object extraction
For `PyClass` types `T`, `&T` and `&mut T` no longer have [`FromPyObject`] implementations.
Instead you should extract `PyRef<T>` or `PyRefMut<T>`, respectively.
If `T` implements `Clone`, you can extract `T` itself.
In addition, you can also extract `&PyCell<T>`, though you rarely need it.

Before:
```ignore
let obj: &PyAny = create_obj();
let obj_ref: &MyClass = obj.extract().unwrap();
let obj_ref_mut: &mut MyClass = obj.extract().unwrap();
```

After:
```rust
# use pyo3::prelude::*;
# use pyo3::types::IntoPyDict;
# #[pyclass] #[derive(Clone)] struct MyClass {}
# #[pymethods] impl MyClass { #[new]fn new() -> Self { MyClass {} }}
# let gil = Python::acquire_gil();
# let py = gil.python();
# let typeobj = py.get_type::<MyClass>();
# let d = [("c", typeobj)].into_py_dict(py);
# let create_obj = || py.eval("c()", None, Some(d)).unwrap();
let obj: &PyAny = create_obj();
let obj_cell: &PyCell<MyClass> = obj.extract().unwrap();
let obj_cloned: MyClass = obj.extract().unwrap(); // extracted by cloning the object
{
    let obj_ref: PyRef<MyClass> = obj.extract().unwrap();
    // we need to drop obj_ref before we can extract a PyRefMut due to Rust's rules of references
}
let obj_ref_mut: PyRefMut<MyClass> = obj.extract().unwrap();
```


#### `#[pyproto]`
Most of the arguments to methods in `#[pyproto]` impls require a
[`FromPyObject`] implementation.
So if your protocol methods take `&T` or `&mut T` (where `T: PyClass`),
please use [`PyRef`] or [`PyRefMut`] instead.

Before:
```rust,compile_fail
# use pyo3::prelude::*;
# use pyo3::class::PySequenceProtocol;
#[pyclass]
struct ByteSequence {
    elements: Vec<u8>,
}
#[pyproto]
impl PySequenceProtocol for ByteSequence {
    fn __concat__(&self, other: &Self) -> PyResult<Self> {
        let mut elements = self.elements.clone();
        elements.extend_from_slice(&other.elements);
        Ok(Self { elements })
    }
}
```

After:
```rust
# use pyo3::prelude::*;
# use pyo3::class::PySequenceProtocol;
#[pyclass]
struct ByteSequence {
    elements: Vec<u8>,
}
#[pyproto]
impl PySequenceProtocol for ByteSequence {
    fn __concat__(&self, other: PyRef<'p, Self>) -> PyResult<Self> {
        let mut elements = self.elements.clone();
        elements.extend_from_slice(&other.elements);
        Ok(Self { elements })
    }
}
```

[`FromPyObject`]: https://docs.rs/pyo3/latest/pyo3/conversion/trait.FromPyObject.html

[`PyCell`]: https://pyo3.rs/master/doc/pyo3/pycell/struct.PyCell.html
[`PyBorrowMutError`]: https://pyo3.rs/master/doc/pyo3/pycell/struct.PyBorrowMutError.html
[`PyRef`]: https://pyo3.rs/master/doc/pyo3/pycell/struct.PyRef.html
[`PyRefMut`]: https://pyo3.rs/master/doc/pyo3/pycell/struct.PyRefMut.html

[`RefCell`]: https://doc.rust-lang.org/std/cell/struct.RefCell.html
