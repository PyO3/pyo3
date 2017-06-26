# Python Class

## Define new class

To define python custom class, rust struct needs to be annotated
with `#[py::class]` attribute.

```rust
extern crate pyo3;

use pyo3::*;

#[py::class]
struct MyClass {
   num: i32,
   debug: bool,
   token: PyToken,
}
```

The above example generates the following implementations for `MyClass` struct

```ignore
impl PyTypeInfo for MyClass { ... }
impl PyTypeObject for MyClass { ... }
impl PyObjectWithToken for MyClass { ... }
impl ToPyObject for MyClass { ... }
impl IntoPyObject for MyClass { ... }
impl ToPyPointer for MyClass { ... }
```

Following implementations `PyObjectWithToken`, `ToPyObject`, `IntoPyObject`, `ToPyPointer`
are generated only if struct contains `PyToken` attribute.

`PyToken` instance available only in `py.init` method.

TODO - continue

## Constructor

By default it is not possible to create instance of custom class from python code.
To declare constructor, you need to define class method and annotate it with `#[new]`
attribute. Only python `__new__` method can be specified, `__init__` is not available.

```rust

#[py::method]
impl MyClass {

     #[new]
     fn __new__(cls: &PyType, ...) -> PyResult<Py<MyClass>> {
         cls.tokne().init(|token| {
             MyClass {
                 num: 10,
                 debug: False,
                 token: token
             }
         })
     }
}
```

Some rules of `new` method

* If no method marked with `#[new]` is declared, object instances can only be created
  from Rust, but not from Python.
* The first parameter is the type object of the class to create.
  This may be the type object of a derived class declared in Python.
* The first parameter implicitly has type `&PyType`.
* For details on `parameter-list`, see the documentation of `Method arguments` section.
* The return type must be `PyResult<T>` for some `T` that implements `IntoPyObject`.
  Usually, `T` will be `MyType`.


## Object properties

Instance's `__dict__` attributes is not supported by pyo3 library. But it is
possible to specify instance get/set descriptors. Descriptor methods can be defined in
`#[methods]` `impl` block only and has to be annotated with `#[getter]` or `[setter]`
attributes. i.e.

```rust

#[methods]
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

#[methods]
impl MyClass {

     #[getter]
     fn get_num(&self) -> PyResult<i32> {
        Ok(self.num)
     }

     #[setter]
     fn set_num(&mut self, value: i32) -> PyResult<()> {
        self.num = value
     }
}
```

In this case property `num` is defined. And it is available from python code as `self.num`.

Also both `#[getter]` and `#[setter]` attributes accepts one parameter.
If parameter is specified, it is used and property name. i.e.

```rust

#[methods]
impl MyClass {

     #[getter(number)]
     fn num(&self) -> PyResult<i32> {
        Ok(self.num)
     }

     #[setter(number)]
     fn set_num(&mut self, value: i32) -> PyResult<()> {
        self.num = value
     }
}
```

In this case property `number` is defined. And it is available from python code as `self.number`.

## Instance methods

To define python compatible method, `impl` block for struct has to be annotated
with `#[py::methods]` attribute. `pyo3` library generates python compatible
wrappers for all functions in this block with some variations, like descriptors,
class method static methods, etc.

```rust

#[methods]
impl MyClass {

     fn method1(&self) -> PyResult<i32> {
        Ok(10)
     }

     fn set_method(&mut self, value: i32) -> PyResult<()> {
        self.num = value
        Ok(())
     }
}
```

Calls to this methods protected by `GIL`, `&self` or `&mut self` can be used.
The return type must be `PyResult<T>` for some `T` that implements `IntoPyObject`.

`Python` parameter can be spefieid as part of method signature, in this case `py` argument
get injected by method wrapper. i.e

```rust

#[methods]
impl MyClass {

     fn method2(&self, py: Python) -> PyResult<i32> {
        Ok(10)
     }
}
```
From python prespective `method2`, in above example, does not accept any arguments.


## Class methods

To specify class method for custom class, method needs to be annotated
with`#[classmethod]` attribute.

```rust

#[methods]
impl MyClass {

     #[classmethod]
     fn cls_method(cls: &PyType) -> PyResult<i32> {
        Ok(10)
     }
}

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

#[methods]
impl MyClass {

     #[staticmethod]
     fn static_method(param1: i32, param2: &str) -> PyResult<i32> {
        Ok(10)
     }
}


## Method arguments

## Class customizations

### Callable object

To specify custom `__call__` method for custom class, call method needs to be annotated
with `#[call]` attribute. Arguments of the method are speficied same as for instance method.


```rust

#[methods]
impl MyClass {

     #[call]
     #[args(args="*")]
     fn __call__(&self, args: &PyTuple) -> PyResult<i32> {
        println!("MyCLS has been called");
        Ok(self.num)
     }
}
