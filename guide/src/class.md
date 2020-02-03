# Python Classes

## Define new class

To define a custom Python class, a Rust struct needs to be annotated with the
`#[pyclass]` attribute.

```rust
# use pyo3::prelude::*;

#[pyclass]
struct MyClass {
   num: i32,
   debug: bool,
}
```

The above example generates implementations for `PyTypeInfo`, `PyTypeObject`
and `PyClass` for `MyClass`.

Specifically, the following implementation is generated:

```rust
use pyo3::prelude::*;

/// Class for demonstration
struct MyClass {
    num: i32,
    debug: bool,
}

impl pyo3::pyclass::PyClassAlloc for MyClass {}

unsafe impl pyo3::PyTypeInfo for MyClass {
    type Type = MyClass;
    type BaseType = pyo3::types::PyAny;
    type ConcreteLayout = pyo3::PyClassShell<Self>;
    type Initializer = pyo3::PyClassInitializer<Self>;

    const NAME: &'static str = "MyClass";
    const MODULE: Option<&'static str> = None;
    const DESCRIPTION: &'static str = "Class for demonstration";
    const FLAGS: usize = 0;

    #[inline]
    fn type_object() -> std::ptr::NonNull<pyo3::ffi::PyTypeObject> {
        use pyo3::type_object::LazyTypeObject;
        static TYPE_OBJECT: LazyTypeObject = LazyTypeObject::new();
        TYPE_OBJECT.get_pyclass_type::<Self>()
    }
}

impl pyo3::pyclass::PyClass for MyClass {
    type Dict = pyo3::pyclass_slots::PyClassDummySlot;
    type WeakRef = pyo3::pyclass_slots::PyClassDummySlot;
}

impl pyo3::IntoPy<PyObject> for MyClass {
    fn into_py(self, py: pyo3::Python) -> pyo3::PyObject {
        pyo3::IntoPy::into_py(pyo3::Py::new(py, self).unwrap(), py)
    }
}

pub struct MyClassGeneratedPyo3Inventory {
    methods: &'static [pyo3::class::PyMethodDefType],
}

impl pyo3::class::methods::PyMethodsInventory for MyClassGeneratedPyo3Inventory {
    fn new(methods: &'static [pyo3::class::PyMethodDefType]) -> Self {
        Self { methods }
    }

    fn get_methods(&self) -> &'static [pyo3::class::PyMethodDefType] {
        self.methods
    }
}

impl pyo3::class::methods::PyMethodsInventoryDispatch for MyClass {
    type InventoryType = MyClassGeneratedPyo3Inventory;
}

pyo3::inventory::collect!(MyClassGeneratedPyo3Inventory);

# let gil = Python::acquire_gil();
# let py = gil.python();
# let cls = py.get_type::<MyClass>();
# pyo3::py_run!(py, cls, "assert cls.__name__ == 'MyClass'")
```

## Add class to module

Custom Python classes can then be added to a module using `add_class`.

```rust
# use pyo3::prelude::*;
# #[pyclass]
# struct MyClass {
#    num: i32,
#    debug: bool,
# }
#[pymodule]
fn mymodule(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<MyClass>()?;
    Ok(())
}
```

## Get Python objects from `pyclass`
You sometimes need to convert your `pyclass` into a Python object in Rust code (e.g., for testing it).

For getting *GIL-bounded* (i.e., with `'py` lifetime) references of `pyclass`,
you can use `PyClassShell<T>`.
Or you can use `Py<T>` directly, for *not-GIL-bounded* references.

### `PyClassShell`
`PyClassShell` represents the actual layout of `pyclass` on the Python heap.

If you want to instantiate `pyclass` in Python and get the reference,
you can use `PyClassShell::new_ref` or `PyClassShell::new_mut`.

```rust
# use pyo3::prelude::*;
# use pyo3::types::PyDict;
# use pyo3::PyClassShell;
#[pyclass]
struct MyClass {
   num: i32,
   debug: bool,
}
let gil = Python::acquire_gil();
let py = gil.python();
let obj = PyClassShell::new_ref(py, MyClass { num: 3, debug: true }).unwrap();
// You can use deref
assert_eq!(obj.num, 3);
let dict = PyDict::new(py);
// You can treat a `&PyClassShell` as a normal Python object
dict.set_item("obj", obj).unwrap();

// return &mut PyClassShell<MyClass>
let obj = PyClassShell::new_mut(py, MyClass { num: 3, debug: true }).unwrap();
obj.num = 5;
```

### `Py`

`Py` is an object wrapper which stores an object longer than the GIL lifetime.

You can use it to avoid lifetime problems.
```rust
# use pyo3::prelude::*;
#[pyclass]
struct MyClass {
   num: i32,
}
fn return_myclass() -> Py<MyClass> {
    let gil = Python::acquire_gil();
    let py = gil.python();
    Py::new(py, MyClass { num: 1 }).unwrap()
}
let gil = Python::acquire_gil();
let obj = return_myclass();
assert_eq!(obj.as_ref(gil.python()).num, 1);
```

## Customizing the class

The `#[pyclass]` macro accepts the following parameters:

* `name=XXX` - Set the class name shown in Python code. By default, the struct name is used as the class name.
* `freelist=XXX` - The `freelist` parameter adds support of free allocation list to custom class.
The performance improvement applies to types that are often created and deleted in a row,
so that they can benefit from a freelist. `XXX` is a number of items for the free list.
* `gc` - Classes with the `gc` parameter participate in Python garbage collection.
If a custom class contains references to other Python objects that can be collected, the `PyGCProtocol` trait has to be implemented.
* `weakref` - Adds support for Python weak references.
* `extends=BaseType` - Use a custom base class. The base `BaseType` must implement `PyTypeInfo`.
* `subclass` - Allows Python classes to inherit from this class.
* `dict` - Adds `__dict__` support, so that the instances of this type have a dictionary containing arbitrary instance variables.
* `module="XXX"` - Set the name of the module the class will be shown as defined in. If not given, the class
  will be a virtual member of the `builtins` module.

## Constructor

By default it is not possible to create an instance of a custom class from Python code.
To declare a constructor, you need to define a method and annotate it with the `#[new]`
attribute. Only Python's `__new__` method can be specified, `__init__` is not available.

```rust
# use pyo3::prelude::*;
#[pyclass]
struct MyClass {
   num: i32,
}

#[pymethods]
impl MyClass {
     #[new]
     fn new(num: i32) -> Self {
         MyClass { num }
     }
}
```

If no method marked with `#[new]` is declared, object instances can only be
created from Rust, but not from Python.

For arguments, see the `Method arguments` section below.

### Return type
Generally, `#[new]` method have to return `T: Into<PyClassInitializer<Self>>` or
`PyResult<T> where T: Into<PyClassInitializer<Self>>`.

For constructors that may fail, you should wrap the return type in a PyResult as well.
Consult the table below to determine which type your constructor should return:

|                             | **Cannot fail**         | **May fail**                      |
|-----------------------------|-------------------------|-----------------------------------|
|**No inheritance**           | `T`                     | `PyResult<T>`                     |
|**Inheritance(T Inherits U)**| `(T, U)`                | `PyResult<(T, U)>`                |
|**Inheritance(General Case)**| `PyClassInitializer<T>` | `PyResult<PyClassInitializer<T>>` |

## Inheritance
By default, `PyAny` is used as the base class. To override this default,
use the `extends` parameter for `pyclass` with the full path to the base class.

For convenience, `(T, U)` implements `Into<PyClassInitializer<T>>` where `U` is the
baseclass of `T`.
But for more deeply nested inheritance, you have to return `PyClassInitializer<T>`
explicitly.

```rust
# use pyo3::prelude::*;
use pyo3::PyClassShell;

#[pyclass]
struct BaseClass {
   val1: usize,
}

#[pymethods]
impl BaseClass {
   #[new]
   fn new() -> Self {
       BaseClass { val1: 10 }
   }

   pub fn method(&self) -> PyResult<usize> {
      Ok(self.val1)
   }
}

#[pyclass(extends=BaseClass)]
struct SubClass {
   val2: usize,
}

#[pymethods]
impl SubClass {
   #[new]
   fn new() -> (Self, BaseClass) {
       (SubClass{ val2: 15}, BaseClass::new())
   }

   fn method2(self_: &PyClassShell<Self>) -> PyResult<usize> {
      self_.get_super().method().map(|x| x * self_.val2)
   }
}

#[pyclass(extends=SubClass)]
struct SubSubClass {
   val3: usize,
}

#[pymethods]
impl SubSubClass {
   #[new]
   fn new() -> PyClassInitializer<Self> {
       PyClassInitializer::from(SubClass::new())
           .add_subclass(SubSubClass{val3: 20})
   }

   fn method3(self_: &PyClassShell<Self>) -> PyResult<usize> {
      let super_ = self_.get_super();
      SubClass::method2(super_).map(|x| x * self_.val3)
   }
}


# let gil = Python::acquire_gil();
# let py = gil.python();
# let subsub = pyo3::PyClassShell::new_ref(py, SubSubClass::new()).unwrap();
# pyo3::py_run!(py, subsub, "assert subsub.method3() == 3000")
```

To access the super class, you can use either of these two ways:
- Use `self_: &PyClassShell<Self>` instead of `self`, and call `get_super()`
- `ObjectProtocol::get_base`
We recommend `PyClassShell` here, since it makes the context much clearer.


If `SubClass` does not provide a baseclass initialization, the compilation fails.
```compile_fail
# use pyo3::prelude::*;
use pyo3::PyClassShell;

#[pyclass]
struct BaseClass {
   val1: usize,
}

#[pyclass(extends=BaseClass)]
struct SubClass {
   val2: usize,
}

#[pymethods]
impl SubClass {
   #[new]
   fn new() -> Self {
       SubClass{ val2: 15}
   }
}
```


## Object properties

Property descriptor methods can be defined in a `#[pymethods]` `impl` block only and have to be
annotated with `#[getter]` and `#[setter]` attributes. For example:

```rust
# use pyo3::prelude::*;
# #[pyclass]
# struct MyClass {
#    num: i32,
# }
#
#[pymethods]
impl MyClass {

     #[getter]
     fn num(&self) -> PyResult<i32> {
        Ok(self.num)
     }
}
```

A getter or setter's function name is used as the property name by default. There are several
ways how to override the name.

If a function name starts with `get_` or `set_` for getter or setter respectively,
the descriptor name becomes the function name with this prefix removed. This is also useful in case of
Rust keywords like `type`
([raw identifiers](https://doc.rust-lang.org/edition-guide/rust-2018/module-system/raw-identifiers.html)
can be used since Rust 2018).

```rust
# use pyo3::prelude::*;
# #[pyclass]
# struct MyClass {
#    num: i32,
# }
#
#[pymethods]
impl MyClass {

     #[getter]
     fn get_num(&self) -> PyResult<i32> {
        Ok(self.num)
     }

     #[setter]
     fn set_num(&mut self, value: i32) -> PyResult<()> {
        self.num = value;
        Ok(())
     }
}
```

In this case, a property `num` is defined and available from Python code as `self.num`.

Both the `#[getter]` and `#[setter]` attributes accept one parameter.
If this parameter is specified, it is used as the property name, i.e.

```rust
# use pyo3::prelude::*;
# #[pyclass]
# struct MyClass {
#    num: i32,
# }
#
#[pymethods]
impl MyClass {

     #[getter(number)]
     fn num(&self) -> PyResult<i32> {
        Ok(self.num)
     }

     #[setter(number)]
     fn set_num(&mut self, value: i32) -> PyResult<()> {
        self.num = value;
        Ok(())
     }
}
```

In this case, the property `number` is defined and available from Python code as `self.number`.

For simple cases where a member variable is just read and written with no side effects, you
can also declare getters and setters in your Rust struct field definition, for example:

```rust
# use pyo3::prelude::*;
#[pyclass]
struct MyClass {
  #[pyo3(get, set)]
  num: i32
}
```

Then it is available from Python code as `self.num`.

## Instance methods

To define a Python compatible method, an `impl` block for your struct has to be annotated with the
`#[pymethods]` attribute. PyO3 generates Python compatible wrappers for all functions in this
block with some variations, like descriptors, class method static methods, etc.

```rust
# use pyo3::prelude::*;
# #[pyclass]
# struct MyClass {
#    num: i32,
# }
#
#[pymethods]
impl MyClass {

     fn method1(&self) -> PyResult<i32> {
        Ok(10)
     }

     fn set_method(&mut self, value: i32) -> PyResult<()> {
        self.num = value;
        Ok(())
     }
}
```

Calls to these methods are protected by the GIL, so both `&self` and `&mut self` can be used.
The return type must be `PyResult<T>` or `T` for some `T` that implements `IntoPy<PyObject>`;
the latter is allowed if the method cannot raise Python exceptions.

A `Python` parameter can be specified as part of method signature, in this case the `py` argument
gets injected by the method wrapper, e.g.

```rust
# use pyo3::prelude::*;
# #[pyclass]
# struct MyClass {
#    num: i32,
#    debug: bool,
# }

#[pymethods]
impl MyClass {
     fn method2(&self, py: Python) -> PyResult<i32> {
        Ok(10)
     }
}
```

From the Python perspective, the `method2` in this example does not accept any arguments.

## Class methods

To create a class method for a custom class, the method needs to be annotated
with the `#[classmethod]` attribute.

```rust
# use pyo3::prelude::*;
# use pyo3::types::PyType;
# #[pyclass]
# struct MyClass {
#    num: i32,
#    debug: bool,
# }

#[pymethods]
impl MyClass {
     #[classmethod]
     fn cls_method(cls: &PyType) -> PyResult<i32> {
        Ok(10)
     }
}
```

Declares a class method callable from Python.

* The first parameter is the type object of the class on which the method is called.
  This may be the type object of a derived class.
* The first parameter implicitly has type `&PyType`.
* For details on `parameter-list`, see the documentation of `Method arguments` section.
* The return type must be `PyResult<T>` or `T` for some `T` that implements `IntoPy<PyObject>`.

## Static methods

To create a static method for a custom class, the method needs to be annotated with the
`#[staticmethod]` attribute. The return type must be `T` or `PyResult<T>` for some `T` that implements
`IntoPy<PyObject>`.

```rust
# use pyo3::prelude::*;
# #[pyclass]
# struct MyClass {
#    num: i32,
#    debug: bool,
# }

#[pymethods]
impl MyClass {
     #[staticmethod]
     fn static_method(param1: i32, param2: &str) -> PyResult<i32> {
        Ok(10)
     }
}
```

## Callable objects

To specify a custom `__call__` method for a custom class, the method needs to be annotated with
the `#[call]` attribute. Arguments of the method are specified as for instance methods.

```rust
# use pyo3::prelude::*;
use pyo3::types::PyTuple;
# #[pyclass]
# struct MyClass {
#    num: i32,
#    debug: bool,
# }

#[pymethods]
impl MyClass {
     #[call]
     #[args(args="*")]
     fn __call__(&self, args: &PyTuple) -> PyResult<i32> {
        println!("MyClass has been called");
        Ok(self.num)
     }
}
```

## Method arguments

By default, PyO3 uses function signatures to determine which arguments are required. Then it scans
the incoming `args` and `kwargs` parameters. If it can not find all required
parameters, it raises a `TypeError` exception. It is possible to override the default behavior
with the `#[args(...)]` attribute. This attribute accepts a comma separated list of parameters in
the form of `attr_name="default value"`. Each parameter has to match the method parameter by name.

Each parameter can be one of the following types:

 * `"*"`: var arguments separator, each parameter defined after `"*"` is a keyword-only parameter.
   Corresponds to python's `def meth(*, arg1.., arg2=..)`.
 * `args="*"`: "args" is var args, corresponds to Python's `def meth(*args)`. Type of the `args`
   parameter has to be `&PyTuple`.
 * `kwargs="**"`: "kwargs" receives keyword arguments, corresponds to Python's `def meth(**kwargs)`.
   The type of the `kwargs` parameter has to be `Option<&PyDict>`.
 * `arg="Value"`: arguments with default value. Corresponds to Python's `def meth(arg=Value)`.
   If the `arg` argument is defined after var arguments, it is treated as a keyword-only argument.
   Note that `Value` has to be valid rust code, PyO3 just inserts it into the generated
   code unmodified.

Example:
```rust
# use pyo3::prelude::*;
use pyo3::types::{PyDict, PyTuple};
#
# #[pyclass]
# struct MyClass {
#    num: i32,
#    debug: bool,
# }
#
#[pymethods]
impl MyClass {
    #[new]
    #[args(num = "-1", debug = "true")]
    fn new(num: i32, debug: bool) -> Self {
        MyClass { num, debug }
    }

    #[args(
        num = "10",
        debug = "true",
        py_args = "*",
        name = "\"Hello\"",
        py_kwargs = "**"
    )]
    fn method(
        &mut self,
        num: i32,
        debug: bool,
        name: &str,
        py_args: &PyTuple,
        py_kwargs: Option<&PyDict>,
    ) -> PyResult<String> {
        self.debug = debug;
        self.num = num;
        Ok(format!(
            "py_args={:?}, py_kwargs={:?}, name={}, num={}, debug={}",
            py_args, py_kwargs, name, self.num, self.debug
        ))
    }

    fn make_change(&mut self, num: i32, debug: bool) -> PyResult<String> {
        self.num = num;
        self.debug = debug;
        Ok(format!("num={}, debug={}", self.num, self.debug))
    }
}
```
N.B. the position of the `"*"` argument (if included) controls the system of handling positional and keyword arguments. In Python:
```python
import mymodule

mc = mymodule.MyClass()
print(mc.method(44, False, "World", 666, x=44, y=55))
print(mc.method(num=-1, name="World"))
print(mc.make_change(44, False))
print(mc.make_change(debug=False, num=-1))
```
Produces output:
```text
py_args=('World', 666), py_kwargs=Some({'x': 44, 'y': 55}), name=Hello, num=44, debug=false
py_args=(), py_kwargs=None, name=World, num=-1, debug=true
num=44, debug=false
num=-1, debug=false
```

## Class customizations

Python's object model defines several protocols for different object behavior, like sequence,
mapping or number protocols. PyO3 defines separate traits for each of them. To provide specific
Python object behavior, you need to implement the specific trait for your struct. Important note,
each protocol implementation block has to be annotated with the `#[pyproto]` attribute.

### Basic object customization

The [`PyObjectProtocol`](https://docs.rs/pyo3/latest/pyo3/class/basic/trait.PyObjectProtocol.html) trait provides several basic customizations.

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

### Garbage Collector Integration

If your type owns references to other Python objects, you will need to
integrate with Python's garbage collector so that the GC is aware of
those references.
To do this, implement the [`PyGCProtocol`](https://docs.rs/pyo3/latest/pyo3/class/gc/trait.PyGCProtocol.html) trait for your struct.
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
        if let Some(ref obj) = self.obj {
            visit.call(obj)?
        }
        Ok(())
    }

    fn __clear__(&mut self) {
        if let Some(obj) = self.obj.take() {
            // Release reference, this decrements ref counter.
            let gil = GILGuard::acquire();
            let py = gil.python();
            py.release(obj);
        }
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
  * `fn __iter__(slf: &mut PyClassShell<Self>) -> PyResult<impl IntoPy<PyObject>>`
  * `fn __next__(slf: &mut PyClassShell<Self>) -> PyResult<Option<impl IntoPy<PyObject>>>`

  Returning `Ok(None)` from `__next__` indicates that that there are no further items.

Example:

```rust
use pyo3::prelude::*;
use pyo3::{PyIterProtocol, PyClassShell};

#[pyclass]
struct MyIterator {
    iter: Box<Iterator<Item = PyObject> + Send>,
}

#[pyproto]
impl PyIterProtocol for MyIterator {
    fn __iter__(slf: &mut PyClassShell<Self>) -> PyResult<Py<MyIterator>> {
        Ok(slf.into())
    }
    fn __next__(slf: &mut PyClassShell<Self>) -> PyResult<Option<PyObject>> {
        Ok(slf.iter.next())
    }
}
```

## Manually implementing pyclass

TODO: Which traits to implement (basically `PyTypeCreate: PyObjectAlloc + PyTypeInfo + PyMethodsProtocol + Sized`) and what they mean.

## How methods are implemented

Users should be able to define a `#[pyclass]` with or without `#[pymethods]`, while PyO3 needs a
trait with a function that returns all methods. Since it's impossible to make the code generation in
pyclass dependent on whether there is an impl block, we'd need to implement the trait on
`#[pyclass]` and override the implementation in `#[pymethods]`, which is to the best of my knowledge
only possible with the specialization feature, which can't be used on stable.

To escape this we use [inventory](https://github.com/dtolnay/inventory), which allows us to collect `impl`s from arbitrary source code by exploiting some binary trick. See [inventory: how it works](https://github.com/dtolnay/inventory#how-it-works) and `pyo3_derive_backend::py_class::impl_inventory` for more details.
