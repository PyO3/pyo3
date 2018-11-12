# Python Class

## Define new class

To define python custom class, rust struct needs to be annotated with `#[pyclass]` attribute.

```rust
# #![feature(specialization)]
# extern crate pyo3;
# use pyo3::prelude::*;

#[pyclass]
struct MyClass {
   num: i32,
   debug: bool,
}
```

The above example generates implementations for `PyTypeInfo` and `PyTypeObject` for `MyClass`.


## Customizing the class

The `#[pyclass]` macro accepts following parameters:

* `name=XXX` - Set the class name shown in python code. By default struct name is used as a class name.
* `freelist=XXX` - `freelist` parameter add support of free allocation list to custom class.
The performance improvement applies to types that are often created and deleted in a row,
so that they can benefit from a freelist. `XXX` is a number of items for free list.
* `gc` - Classes with the `gc` parameter
participate in python garbage collector. If a custom class contains references to other
python object that can be collected, the `PyGCProtocol` trait has to be implemented.
* `weakref` - adds support for python weak references
* `extends=BaseType` - use a custom base class. The base BaseType must implement `PyTypeInfo`.
* `subclass` - Allows Python classes to inherit from this class
* `dict` - adds `__dict__` support, the instances of this type have a dictionary containing instance variables. (Incomplete, see [#123](https://github.com/PyO3/pyo3/issues/123))


## Constructor

By default it is not possible to create an instance of a custom class from python code.
To declare a constructor, you need to define a class method and annotate it with `#[new]`
attribute. Only the python `__new__` method can be specified, `__init__` is not available.

```rust
# #![feature(specialization)]
#
# extern crate pyo3;
# use pyo3::prelude::*;
# use pyo3::PyRawObject;

#[pyclass]
struct MyClass {
   num: i32,
}

#[pymethods]
impl MyClass {

     #[new]
     fn __new__(obj: &PyRawObject, num: i32) -> PyResult<()> {
         obj.init(||   {
             MyClass {
                 num,
             }
         })
     }
}
```

Rules for the `new` method:

* If no method marked with `#[new]` is declared, object instances can only be created
  from Rust, but not from Python.
* The first parameter is the raw object and the custom `new` method must initialize the object
  with an instance of the struct using `init` method. The type of the object may be the type object of
  a derived class declared in Python.
* The first parameter implicitly has type `&PyRawObject`.
* For details on `parameter-list`, see the documentation of `Method arguments` section.
* The return type must be `PyResult<T>` for some `T` that implements `IntoPyObject`. Usually, `T` will be `MyType`.


## Inheritance

By default `PyObject` is used as default base class. To override default base class
`base` parameter for `class` needs to be used. Value is full path to base class.
`__new__` method accepts `PyRawObject` object. `obj` instance must be initialized
with value of custom class struct. Subclass must call parent's `__new__` method.

```rust
# #![feature(specialization)]
# extern crate pyo3;
# use pyo3::prelude::*;
# use pyo3::PyRawObject;
#[pyclass]
struct BaseClass {
   val1: usize,
}

#[pymethods]
impl BaseClass {
   #[new]
   fn __new__(obj: &PyRawObject) -> PyResult<()> {
       obj.init(||   BaseClass{ val1: 10 })
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
   fn __new__(obj: &PyRawObject) -> PyResult<()> {
       obj.init(||   SubClass{ val2: 10 });
       BaseClass::__new__(obj)
   }

   fn method2(&self) -> PyResult<()> {
       self.get_base().method()
   }
}
```

`ObjectProtocol` trait provides `get_base()` method. It returns reference to instance of
base class.


## Object properties

Descriptor methods can be defined in
`#[pymethods]` `impl` block only and has to be annotated with `#[getter]` or `[setter]`
attributes. i.e.

```rust
# #![feature(specialization)]
# extern crate pyo3;
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

Getter or setter function's name is used as property name by default. There are several
ways how to override name.

If function name starts with `get_` or `set_` for getter or setter respectively.
Descriptor name becomes function name with prefix removed. This is useful in case os
rust's special keywords like `type`.

```rust
# #![feature(specialization)]
# extern crate pyo3;
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

In this case property `num` is defined. And it is available from python code as `self.num`.

Also both `#[getter]` and `#[setter]` attributes accepts one parameter.
If parameter is specified, it is used and property name. i.e.

```rust
# #![feature(specialization)]
# extern crate pyo3;
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

In this case property `number` is defined. And it is available from python code as `self.number`.

For simple cases you can also define getters and setters in your Rust struct field definition, for example:

```rust
# #![feature(specialization)]
# extern crate pyo3;
# use pyo3::prelude::*;
#[pyclass]
struct MyClass {
  #[prop(get, set)]
  num: i32
}
```

Then it is available from Python code as `self.num`.

## Instance methods

To define python compatible method, `impl` block for struct has to be annotated
with `#[pymethods]` attribute. `pyo3` library generates python compatible
wrappers for all functions in this block with some variations, like descriptors,
class method static methods, etc.

```rust
# #![feature(specialization)]
# extern crate pyo3;
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

Calls to this methods protected by `GIL`, `&self` or `&mut self` can be used.
The return type must be `PyResult<T>` for some `T` that implements `IntoPyObject`.

`Python` parameter can be specified as part of method signature, in this case `py` argument
get injected by method wrapper. i.e

```rust
# #![feature(specialization)]
# extern crate pyo3;
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

From python perspective `method2`, in above example, does not accept any arguments.

## Class methods

To specify class method for custom class, method needs to be annotated
with`#[classmethod]` attribute.

```rust
# #![feature(specialization)]
# extern crate pyo3;
# use pyo3::prelude::*;
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
* The return type must be `PyResult<T>` for some `T` that implements `IntoPyObject`.

## Static methods

To specify class method for custom class, method needs to be annotated
with `#[staticmethod]` attribute. The return type must be `PyResult<T>`
for some `T` that implements `IntoPyObject`.

```rust
# #![feature(specialization)]
# extern crate pyo3;
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

## Callable object

To specify custom `__call__` method for custom class, call method needs to be annotated
with `#[call]` attribute. Arguments of the method are specified same as for instance method.

```rust
# #![feature(specialization)]
# extern crate pyo3;
# use pyo3::prelude::*;
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

By default pyo3 library uses function signature to determine which arguments are required.
Then it scans incoming `args` parameter and then incoming `kwargs` parameter. If it can not
find all required parameters, it raises `TypeError` exception.
It is possible to override default behavior with `#[args(...)]` attribute. `args` attribute
accept comma separated list of parameters in form `attr_name="default value"`. Each parameter
has to match method parameter by name.

Each parameter could one of following type:

 * "\*": var arguments separator, each parameter defined after "*" is keyword only parameters.
   corresponds to python's `def meth(*, arg1.., arg2=..)`
 * args="\*": "args" is var args, corresponds to python's `def meth(*args)`. Type of `args`
   parameter has to be `&PyTuple`.
 * kwargs="\*\*": "kwargs" is keyword arguments, corresponds to python's `def meth(**kwargs)`.
   Type of `kwargs` parameter has to be `Option<&PyDict>`.
 * arg="Value": arguments with default value. corresponds to python's `def meth(arg=Value)`.
   if `arg` argument is defined after var arguments it is treated as keyword argument.
   Note that `Value` has to be valid rust code, pyo3 just inserts it into generated
   code unmodified.

Example:
```rust
# #![feature(specialization)]
# extern crate pyo3;
# use pyo3::prelude::*;
#
# #[pyclass]
# struct MyClass {
#    num: i32,
#    debug: bool,
# }
#
#[pymethods]
impl MyClass {
    #[args(arg1=true, args="*", arg2=10, kwargs="**")]
    fn method(&self, arg1: bool, args: &PyTuple, arg2: i32, kwargs: Option<&PyDict>) -> PyResult<i32> {
        Ok(1)
    }
}
```


## Class customizations

Python object model defines several protocols for different object behavior,
like sequence, mapping or number protocols. pyo3 library defines separate trait for each
of them. To provide specific python object behavior you need to implement specific trait
for your struct. Important note, each protocol implementation block has to be annotated
with `#[pyproto]` attribute.

### Basic object customization

[`PyObjectProtocol`](https://docs.rs/pyo3/0.2.7/class/basic/trait.PyObjectProtocol.html) trait provide several basic customizations.

#### Attribute access

To customize object attribute access define following methods:

  * `fn __getattr__(&self, name: FromPyObject) -> PyResult<impl IntoPyObject>`
  * `fn __setattr__(&mut self, name: FromPyObject, value: FromPyObject) -> PyResult<()>`
  * `fn __delattr__(&mut self, name: FromPyObject) -> PyResult<()>`

Each methods corresponds to python's `self.attr`, `self.attr = value` and `del self.attr` code.

#### String Conversions

  * `fn __repr__(&self) -> PyResult<impl ToPyObject<ObjectType=PyString>>`
  * `fn __str__(&self) -> PyResult<impl ToPyObject<ObjectType=PyString>>`

    Possible return types for `__str__` and `__repr__` are `PyResult<String>` or `PyResult<PyString>`.
    In Python 2.7, Unicode strings returned by `__str__` and `__repr__` will be converted to byte strings
    by the Python runtime, which results in an exception if the string contains non-ASCII characters.

  * `fn __bytes__(&self) -> PyResult<PyBytes>`

    On Python 3.x, provides the conversion to `bytes`.
    On Python 2.7, `__bytes__` is allowed but has no effect.

  * `fn __unicode__(&self) -> PyResult<PyUnicode>`

    On Python 2.7, provides the conversion to `unicode`.
    On Python 3.x, `__unicode__` is allowed but has no effect.

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
    This method works for both python 3 and python 2,
    even on Python 2.7 where the Python spelling was `__nonzero__`.

### Garbage Collector Integration

If your type owns references to other python objects, you will need to
integrate with Python's garbage collector so that the GC is aware of
those references.
To do this, implement [`PyGCProtocol`](https://docs.rs/pyo3/0.2.7/class/gc/trait.PyGCProtocol.html) trait for your struct.
It includes two methods `__traverse__` and `__clear__`.
These correspond to the slots `tp_traverse` and `tp_clear` in the Python C API.
`__traverse__` must call `visit.call()` for each reference to another python object.
`__clear__` must clear out any mutable references to other python objects
(thus breaking reference cycles). Immutable references do not have to be cleared,
as every cycle must contain at least one mutable reference.
Example:
```rust
#![feature(specialization)]
extern crate pyo3;

use pyo3::prelude::*;


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
          self.py().release(obj);
        }
    }
}
```

Special protocol trait implementation has to be annotated with `#[pyproto]` attribute.

It is also possible to enable gc for custom class using `gc` parameter for `class` annotation.
i.e. `#[pyclass(gc)]`. In that case instances of custom class participate in python garbage
collector, and it is possible to track them with `gc` module methods.

### Iterator Types

Iterators can be defined using the
[`PyIterProtocol`](https://docs.rs/pyo3/0.2.7/class/iter/trait.PyIterProtocol.html) trait.
It includes two methods `__iter__` and `__next__`:
  * `fn __iter__(&mut self) -> PyResult<impl IntoPyObject>`
  * `fn __next__(&mut self) -> PyResult<Option<impl IntoPyObject>>`

  Returning `Ok(None)` from `__next__` indicates that that there are no further items.

Example:

```rust
#![feature(specialization)]

extern crate pyo3;

use pyo3::prelude::*;

#[pyclass]
struct MyIterator {
    iter: Box<Iterator<Item=PyObject> + Send>,
}

#[pyproto]
impl PyIterProtocol for MyIterator {

    fn __iter__(&mut self) -> PyResult<PyObject> {
        Ok(self.into())
    }
    fn __next__(&mut self) -> PyResult<Option<PyObject>> {
        Ok(self.iter.next())
    }
}
```
