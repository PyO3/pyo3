# Appendix B: Migration Guides for major version changes

## from 0.8.* to 0.9

### `#[new]` interface
[`PyRawObject`](https://docs.rs/pyo3/0.8.5/pyo3/type_object/struct.PyRawObject.html)
is now removed and our syntax for constructor changed.

Before:
```compile_fail
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
```
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
PyO3 0.9 introduces [`PyCell`](https://pyo3.rs/master/doc/pyo3/pycell/struct.PyCell.html), which is
a [`RefCell`](https://doc.rust-lang.org/std/cell/struct.RefCell.html) like object wrapper
for dynamically ensuring
[Rust's rule of Reference](https://doc.rust-lang.org/book/ch04-02-references-and-borrowing.html#the-rules-of-references).

For `#[pymethods]` or `#[pyfunction]`s, `PyCell` works without any change.
Just throw errors when there happens invalid borrowing.

Here is an example.
```
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
# except Exception as e:
#    isinstance(e, borrow_mut_err)
# ");
```
`Names` has `merge` method, which takes `&mut self` and `&mut Self`.
Given this `#[pyclass]`, calling `names.merge(names)` in Python raises `PyBorrowMutError` exception,
since it requires two mutable borrows of `names`,

However, for `#[pyproto]` and some functions, you need to manually fix codes.

#### Object creation
We could use the older `PyRef` and `PyRefMut` for object creation, but now they are just
reference wrappers for `PyCell`.
Use `PyCell::new` instead.

Before:
```compile_fail
# use pyo3::prelude::*;
# #[pyclass]
# struct MyClass {}
let gil = Python::acquire_gil();
let py = gil.python();
let obj_ref = PyRef::new(py, MyClass {}).unwrap();
```

After:
```
# use pyo3::prelude::*;
# #[pyclass]
# struct MyClass {}
let gil = Python::acquire_gil();
let py = gil.python();
let obj = PyCell::new(py, MyClass {}).unwrap();
let obj_ref = obj.borrow();
```

#### Object extraction
Now for `T: PyClass`, `&T` and `&mut T` don't have `FromPyObject` implementation.
Instead, you can use `&PyCell`, `PyRef`, and `PyRefMut` for object extraction.

Before:
```ignore
let obj: &PyAny = create_obj();
let obj_ref: &MyClass = obj.extract().unwrap();
let obj_ref_mut: &mut MyClass = obj.extract().unwrap();
```

After:
```
# use pyo3::prelude::*;
# use pyo3::types::{PyAny, IntoPyDict};
# #[pyclass] struct MyClass {}
# #[pymethods] impl MyClass { #[new]fn new() -> Self { MyClass {} }}
# let gil = Python::acquire_gil();
# let py = gil.python();
# let typeobj = py.get_type::<MyClass>();
# let d = [("c", typeobj)].into_py_dict(py);
# let create_obj = || py.eval("c()", None, Some(d)).unwrap();
let obj: &PyAny = create_obj();
let obj_cell: &PyCell<MyClass> = obj.extract().unwrap();
{
    let obj_ref: PyRef<MyClass> = obj.extract().unwrap();
    // we need to drop obj_ref before taking RefMut
}
let obj_ref_mut: PyRefMut<MyClass> = obj.extract().unwrap();
```


#### `#[pyproto]`
Most of `#[pyproto]` arguments requires [`FromPyObject`] implementation.
So if your protocol methods take `&T` or `&mut T`(where `T: PyClass`),
please use `PyRef` or `PyRefMut` instead.

Before:
```compile_fail
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
```
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
