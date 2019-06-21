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

The above example generates implementations for `PyTypeInfo` and `PyTypeObject` for `MyClass`.

## Get Python objects from `pyclass`

You can use `pyclass`es like normal rust structs.

However, if instantiated normally, you can't treat `pyclass`es as Python objects.

To get a Python object which includes `pyclass`, we have to use some special methods.

### `PyRef`

`PyRef` is a special reference, which ensures that the referred struct is a part of
a Python object, and you are also holding the GIL.

You can get an instance of `PyRef` by `PyRef::new`, which does 3 things:
1. Allocates a Python object in the Python heap
2. Copies the Rust struct into the Python object
3. Returns a reference to it

You can use `PyRef` just like `&T`, because it implements `Deref<Target=T>`.
```rust
# use pyo3::prelude::*;
# use pyo3::types::PyDict;
#[pyclass]
struct MyClass {
   num: i32,
   debug: bool,
}
let gil = Python::acquire_gil();
let py = gil.python();
let obj = PyRef::new(py, MyClass { num: 3, debug: true }).unwrap();
assert_eq!(obj.num, 3);
let dict = PyDict::new(py);
// You can treat a `PyRef` as a Python object
dict.set_item("obj", obj).unwrap();
```

### `PyRefMut`

`PyRefMut` is a mutable version of `PyRef`.
```rust
# use pyo3::prelude::*;
#[pyclass]
struct MyClass {
   num: i32,
   debug: bool,
}
let gil = Python::acquire_gil();
let py = gil.python();
let mut obj = PyRefMut::new(py, MyClass { num: 3, debug: true }).unwrap();
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
# use pyo3::PyRawObject;
#[pyclass]
struct MyClass {
   num: i32,
}

#[pymethods]
impl MyClass {

     #[new]
     fn new(obj: &PyRawObject, num: i32) {
         obj.init({
             MyClass {
                 num,
             }
         });
     }
}
```

Rules for the `new` method:

* If no method marked with `#[new]` is declared, object instances can only be created
  from Rust, but not from Python.
* The first parameter is the raw object and the custom `new` method must initialize the object
  with an instance of the struct using the `init` method. The type of the object may be the type object of
  a derived class declared in Python.
* The first parameter must have type `&PyRawObject`.
* For details on the parameter list, see the `Method arguments` section below.
* The return value must be `T` or `PyResult<T>` where `T` is ignored, so it can
  be just `()` as in the example above.


## Inheritance

By default, `PyObject` is used as the base class. To override this default,
use the `extends` parameter for `pyclass` with the full path to the base class.
The `new` method of subclasses must call their parent's `new` method.

```rust,ignore
# use pyo3::prelude::*;
# use pyo3::PyRawObject;
#[pyclass]
struct BaseClass {
   val1: usize,
}

#[pymethods]
impl BaseClass {
   #[new]
   fn new(obj: &PyRawObject) {
       obj.init(BaseClass { val1: 10 });
   }

   pub fn method(&self) -> PyResult<()> {
      Ok(())
   }
}

#[pyclass(extends=BaseClass)]
struct SubClass {
   val2: usize,
}

#[pymethods]
impl SubClass {
   #[new]
   fn new(obj: &PyRawObject) {
       obj.init(SubClass { val2: 10 });
       BaseClass::new(obj);
   }

   fn method2(&self) -> PyResult<()> {
      self.get_base().method()
   }
}
```

The `ObjectProtocol` trait provides a `get_base()` method, which returns a reference
to the instance of the base struct.


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
The return type must be `PyResult<T>` or `T` for some `T` that implements `IntoPyObject`;
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
* The return type must be `PyResult<T>` or `T` for some `T` that implements `IntoPyObject`.

## Static methods

To create a static method for a custom class, the method needs to be annotated with the
`#[staticmethod]` attribute. The return type must be `T` or `PyResult<T>` for some `T` that implements
`IntoPyObject`.

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
    #[args(arg1=true, args="*", arg2=10, args3="\"Hello\"", kwargs="**")]
    fn method(&self, arg1: bool, args: &PyTuple, arg2: i32, arg3: &str, kwargs: Option<&PyDict>) -> PyResult<i32> {
        Ok(1)
    }
}
```


## Class customizations

Python's object model defines several protocols for different object behavior, like sequence,
mapping or number protocols. PyO3 defines separate traits for each of them. To provide specific
Python object behavior, you need to implement the specific trait for your struct. Important note,
each protocol implementation block has to be annotated with the `#[pyproto]` attribute.

### Basic object customization

The [`PyObjectProtocol`](https://docs.rs/pyo3/0.7.0/pyo3/class/basic/trait.PyObjectProtocol.html) trait provides several basic customizations.

#### Attribute access

To customize object attribute access, define the following methods:

  * `fn __getattr__(&self, name: FromPyObject) -> PyResult<impl IntoPyObject>`
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
To do this, implement the [`PyGCProtocol`](https://docs.rs/pyo3/0.7.0/pyo3/class/gc/trait.PyGCProtocol.html) trait for your struct.
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

It is also possible to enable GC for custom class using the `gc` parameter of the `pyclass` attribute.
i.e. `#[pyclass(gc)]`. In that case instances of custom class participate in Python garbage
collection, and it is possible to track them with `gc` module methods.

### Iterator Types

Iterators can be defined using the
[`PyIterProtocol`](https://docs.rs/pyo3/0.7.0/pyo3/class/iter/trait.PyIterProtocol.html) trait.
It includes two methods `__iter__` and `__next__`:
  * `fn __iter__(slf: PyRefMut<Self>) -> PyResult<impl IntoPyObject>`
  * `fn __next__(slf: PyRefMut<Self>) -> PyResult<Option<impl IntoPyObject>>`

  Returning `Ok(None)` from `__next__` indicates that that there are no further items.

Example:

```rust
extern crate pyo3;

use pyo3::prelude::*;
use pyo3::PyIterProtocol;

#[pyclass]
struct MyIterator {
    iter: Box<Iterator<Item = PyObject> + Send>,
}

#[pyproto]
impl PyIterProtocol for MyIterator {
    fn __iter__(slf: PyRefMut<Self>) -> PyResult<Py<MyIterator>> {
        Ok(slf.into())
    }
    fn __next__(mut slf: PyRefMut<Self>) -> PyResult<Option<PyObject>> {
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
