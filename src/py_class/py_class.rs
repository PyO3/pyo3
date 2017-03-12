// Copyright (c) 2016 Daniel Grunwald
//
// Permission is hereby granted, free of charge, to any person obtaining a copy of this
// software and associated documentation files (the "Software"), to deal in the Software
// without restriction, including without limitation the rights to use, copy, modify, merge,
// publish, distribute, sublicense, and/or sell copies of the Software, and to permit persons
// to whom the Software is furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in all copies or
// substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED,
// INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR
// PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE
// FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR
// OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER
// DEALINGS IN THE SOFTWARE.

/**
Defines new python extension class.
A `py_class!` macro invocation generates code that declares a new Python class.
Additionally, it generates a Rust struct of the same name, which allows accessing
instances of that Python class from Rust.

# Syntax
`py_class!(pub class MyType |py| { ... })`

* `pub` makes the generated Rust struct visible outside the current module. It has no effect on the visibility from Python.
* `MyType` is the name of the Python class.
* `py` is an identifier that will be made available as a variable of type `Python`
in all function bodies.
* `{ ... }` is the class body, described in more detail below.

# Example
```
#[macro_use] extern crate cpython;
use cpython::{Python, PyResult, PyDict};

py_class!(class MyType |py| {
    data number: i32;
    def __new__(_cls, arg: i32) -> PyResult<MyType> {
        MyType::create_instance(py, arg)
    }
    def half(&self) -> PyResult<i32> {
        println!("half() was called with self={:?}", self.number(py));
        Ok(self.number(py) / 2)
    }
});

fn main() {
    let gil = Python::acquire_gil();
    let py = gil.python();
    let dict = PyDict::new(py);
    dict.set_item(py, "MyType", py.get_type::<MyType>()).unwrap();
    py.run("assert MyType(42).half() == 21", None, Some(&dict)).unwrap();
}
```

# Generated Rust type

The above example generates the following Rust type:

```ignore
struct MyType { ... }

impl ToPythonObject for MyType { ... }
impl PythonObject for MyType { ... }
impl PythonObjectWithCheckedDowncast for MyType { ... }
impl PythonObjectWithTypeObject for MyType { ... }
impl PythonObjectFromPyClassMacro for MyType { ... }

impl MyType {
    fn create_instance(py: Python, number: i32) -> PyResult<MyType> { ... }

    // data accessors
    fn number<'a>(&'a self, py: Python<'a>) -> &'a i32 { ... }

    // functions callable from python
    pub fn __new__(_cls: &PyType, py: Python, arg: i32) -> PyResult<MyType> {
        MyType::create_instance(py, arg)
    }

    pub fn half(&self, py: Python) -> PyResult<i32> {
        println!("half() was called with self={:?}", self.number(py));
        Ok(self.number(py) / 2)
    }
}
```

* The generated type implements a number of traits from the `cpython` crate.
* The inherent `create_instance` method can create new Python objects
  given the values for the data fields.
* Private accessors functions are created for the data fields.
* All functions callable from Python are also exposed as public Rust functions.
* To convert from `MyType` to `PyObject`, use `as_object()` or `into_object()` (from the `PythonObject` trait).
* To convert `PyObject` to `MyType`, use `obj.cast_as::<MyType>(py)` or `obj.cast_into::<MyType>(py)`.

# py_class body
The body of a `py_class!` supports the following definitions:

## Data declarations
`data data_name: data_type;`

Declares a data field within the Python class.
Used to store Rust data directly in the Python object instance.

Because Python code can pass all Python objects to other threads,
`data_type` must be `Send + 'static`.

Because Python object instances can be freely shared (Python has no concept of "ownership"),
data fields cannot be declared as `mut`.
If mutability is required, you have to use interior mutability (`Cell` or `RefCell`).

If data members are used to store references to other Python objects, make sure
to read the section "Garbage Collector Integration".

Data declarations are not accessible from Python.
On the Rust side, data is accessed through the automatically generated accessor functions:
```ignore
impl MyType {
    fn data_name<'a>(&'a self, py: Python<'a>) -> &'a data_type { ... }
}
```

## Instance methods
`def method_name(&self, parameter-list) -> PyResult<...> { ... }`

Declares an instance method callable from Python.

* Because Python objects are potentially shared, the self parameter must always
  be a shared reference (`&self`).
* For details on `parameter-list`, see the documentation of `py_argparse!()`.
* The return type must be `PyResult<T>` for some `T` that implements `ToPyObject`.

## Class methods
`@classmethod def method_name(cls, parameter-list) -> PyResult<...> { ... }`

Declares a class method callable from Python.

* The first parameter is the type object of the class on which the method is called.
  This may be the type object of a derived class.
* The first parameter implicitly has type `&PyType`. This type must not be explicitly specified.
* For details on `parameter-list`, see the documentation of `py_argparse!()`.
* The return type must be `PyResult<T>` for some `T` that implements `ToPyObject`.

## Static methods
`@staticmethod def method_name(parameter-list) -> PyResult<...> { ... }`

Declares a static method callable from Python.

* For details on `parameter-list`, see the documentation of `py_argparse!()`.
* The return type must be `PyResult<T>` for some `T` that implements `ToPyObject`.

## __new__
`def __new__(cls, parameter-list) -> PyResult<...> { ... }`

Declares a constructor method callable from Python.

* If no `__new__` method is declared, object instances can only be created from Rust (via `MyType::create_instance`),
  but not from Python.
* The first parameter is the type object of the class to create.
  This may be the type object of a derived class declared in Python.
* The first parameter implicitly has type `&PyType`. This type must not be explicitly specified.
* For details on `parameter-list`, see the documentation of `py_argparse!()`.
* The return type must be `PyResult<T>` for some `T` that implements `ToPyObject`.
  Usually, `T` will be `MyType`.

## Garbage Collector Integration

If your type owns references to other python objects, you will need to
integrate with Python's garbage collector so that the GC is aware of
those references.
To do this, implement the special member functions `__traverse__` and `__clear__`.
These correspond to the slots `tp_traverse` and `tp_clear` in the Python C API.

`__traverse__` must call `visit.call()` for each reference to another python object.

`__clear__` must clear out any mutable references to other python objects
(thus breaking reference cycles). Immutable references do not have to be cleared,
as every cycle must contain at least one mutable reference.

Example:

```
#[macro_use] extern crate cpython;
use std::{mem, cell};
use cpython::{PyObject, PyDrop};

py_class!(class ClassWithGCSupport |py| {
    data obj: cell::RefCell<Option<PyObject>>;

    def __traverse__(&self, visit) {
        if let Some(ref obj) = *self.obj(py).borrow() {
            try!(visit.call(obj))
        }
        Ok(())
    }

    def __clear__(&self) {
        let old_obj = mem::replace(&mut *self.obj(py).borrow_mut(), None);
        // Release reference only after the mutable borrow has expired,
        // see Caution note below.
        old_obj.release_ref(py);
    }
});
# fn main() {}
```

Caution: `__traverse__` may be called by the garbage collector:

  * during any python operation that takes a `Python` token as argument
  * indirectly from the `PyObject` (or derived type) `Drop` implementation
  * if your code releases the GIL, at any time by other threads.

If you are using `RefCell<PyObject>`, you must not perform any of the above
operations while your code holds a mutable borrow, or you may cause the borrow
in `__traverse__` to panic.

This is why the example above uses the `mem::replace`/`release_ref` dance:
`release_ref` (or the implicit `Drop`) can only be called safely in a separate
statement, after the mutable borrow on the `RefCell` has expired.

Note that this restriction applies not only to `__clear__`, but to all methods
that use `RefCell::borrow_mut`.

## Iterator Types

Iterators can be defined using the Python special methods `__iter__` and `__next__`:

  * `def __iter__(&self) -> PyResult<impl ToPyObject>`
  * `def __next__(&self) -> PyResult<Option<impl ToPyObject>>`

    Returning `Ok(None)` from `__next__` indicates that that there are no further items.

Example:

```
#[macro_use] extern crate cpython;
use std::cell::RefCell;
use cpython::{PyObject, PyClone, PyResult};

py_class!(class MyIterator |py| {
    data iter: RefCell<Box<Iterator<Item=PyObject> + Send>>;

    def __iter__(&self) -> PyResult<Self> {
        Ok(self.clone_ref(py))
    }

    def __next__(&self) -> PyResult<Option<PyObject>> {
        Ok(self.iter(py).borrow_mut().next())
    }
});
# fn main() {}
```

## String Conversions

  * `def __repr__(&self) -> PyResult<impl ToPyObject<ObjectType=PyString>>`
  * `def __str__(&self) -> PyResult<impl ToPyObject<ObjectType=PyString>>`

    Possible return types for `__str__` and `__repr__` are `PyResult<String>` or `PyResult<PyString>`.

    In Python 2.7, Unicode strings returned by `__str__` and `__repr__` will be converted to byte strings
    by the Python runtime, which results in an exception if the string contains non-ASCII characters.

  * `def __bytes__(&self) -> PyResult<PyBytes>`

    On Python 3.x, provides the conversion to `bytes`.
    On Python 2.7, `__bytes__` is allowed but has no effect.

  * `def __unicode__(&self) -> PyResult<PyUnicode>`

    On Python 2.7, provides the conversion to `unicode`.
    On Python 3.x, `__unicode__` is allowed but has no effect.

  * `def __format__(&self, format_spec: &str) -> PyResult<impl ToPyObject<ObjectType=PyString>>`

    Special method that is used by the `format()` builtin and the `str.format()` method.
    Possible return types are `PyResult<String>` or `PyResult<PyString>`.

## Comparison operators

  * `def __richcmp__(&self, other: impl FromPyObject, op: CompareOp) -> PyResult<impl ToPyObject>`

    Overloads Python comparison operations (`==`, `!=`, `<`, `<=`, `>`, and `>=`).
    The `op` argument indicates the comparison operation being performed.
    The return type will normally be `PyResult<bool>`, but any Python object can be returned.

    If `other` is not of the type specified in the signature, the generated code will
    automatically `return NotImplemented`.

  * `def __hash__(&self) -> PyResult<impl PrimInt>`

    Objects that compare equal must have the same hash value.
    The return type must be `PyResult<T>` where `T` is one of Rust's primitive integer types.

## Emulating Container Types

  * `def __len__(&self) -> PyResult<usize>`

    Called by the built-in Python function `len()`.

  * `def __length_hint__(&self) -> PyResult<usize>`

    Should return an estimated length for the object.
    This method is purely an optimization and is never required for correctness.

    `__length_hint__` is new in Python 3.4; older versions will ignore the method.

  * `def __getitem__(&self, key: impl FromPyObject) -> PyResult<impl ToPyObject>`

    Called by the Python subscript operator `self[key]`.

  * `def __setitem__(&self, key: impl FromPyObject, value: impl FromPyObject) -> PyResult<()>`

    Called by Python `self[key] = value`.

  * `def __delitem__(&self, key: impl FromPyObject) -> PyResult<()>`

    Called by Python `del self[key]`.

  * `def __reversed__(&self) -> PyResult<impl ToPyObject>`

    Called by the `reversed()` built-in.
    It should return a new iterator object that iterates over all the objects in the container in reverse order.

  * `def __contains__(&self, item: impl FromPyObject) -> PyResult<bool>`

    Called by Python `item in self`.
    For mapping types, this should consider the keys of the mapping rather than the values
    or the key-item pairs.

    If extraction of the `item` parameter fails with `TypeError`,
    `__contains__` will return `Ok(false)`.

## Arithmetic methods

  * `def __add__(lhs, rhs) -> PyResult<impl ToPyObject>`
  * `def __sub__(lhs, rhs) -> PyResult<impl ToPyObject>`
  * `def __mul__(lhs, rhs) -> PyResult<impl ToPyObject>`
  * `def __lshift__(lhs, rhs) -> PyResult<impl ToPyObject>`
  * `def __rshift__(lhs, rhs) -> PyResult<impl ToPyObject>`
  * `def __and__(lhs, rhs) -> PyResult<impl ToPyObject>`
  * `def __xor__(lhs, rhs) -> PyResult<impl ToPyObject>`
  * `def __or__(lhs, rhs) -> PyResult<impl ToPyObject>`

    The parameters `lhs` and `rhs` must not be given an explicit type.
    Within the method bodies, both parameters will implicitly have type `&PyObject`.

    There are no separate "reversed" versions of these methods (`__radd__()`, etc.)
    Instead, if the first operand cannot perform the operation,
    the same method of the second operand is called, with the operands in the same order.

    This means that you can't rely on the first parameter of these methods being `self`
    or being the correct type, and you should test the types of both operands before deciding what to do.
    If you can't handle the combination of types you've been given,
    you should return `Ok(py.NotImplemented())`.

  * `def __iadd__(&self, other: impl FromPyObject) -> PyResult<impl ToPyObject>`
  * `def __isub__(&self, other: impl FromPyObject) -> PyResult<impl ToPyObject>`
  * `def __imul__(&self, other: impl FromPyObject) -> PyResult<impl ToPyObject>`
  * `def __imatmul__(&self, other: impl FromPyObject) -> PyResult<impl ToPyObject>`
  * `def __itruediv__(&self, other: impl FromPyObject) -> PyResult<impl ToPyObject>`
  * `def __ifloordiv__(&self, other: impl FromPyObject) -> PyResult<impl ToPyObject>`
  * `def __imod__(&self, other: impl FromPyObject) -> PyResult<impl ToPyObject>`
  * `def __ilshift__(&self, other: impl FromPyObject) -> PyResult<impl ToPyObject>`
  * `def __irshift__(&self, other: impl FromPyObject) -> PyResult<impl ToPyObject>`
  * `def __iand__(&self, other: impl FromPyObject) -> PyResult<impl ToPyObject>`
  * `def __ixor__(&self, other: impl FromPyObject) -> PyResult<impl ToPyObject>`
  * `def __ior__(&self, other: impl FromPyObject) -> PyResult<impl ToPyObject>`

    Handles inplace operations if possible, falling back to the non-inplace versions.
    These methods must return a new reference! In the common case of returning the
    same (mutated) object, you will want to return `Ok(self.clone_ref(py))`.

    If you can't handle the combination of types you've been given,
    you should return `Ok(py.NotImplemented())`.

## Context Manager

  * `def __enter__(&self) -> PyResult<impl ToPyObject>`
  * `def __exit__(&self, ty: Option<PyType>, value: PyObject, traceback: PyObject) -> PyResult<bool>`

## Other Special Methods

  * `def __bool__(&self) -> PyResult<bool>`

    Determines the "truthyness" of the object.

    Note that `py_class!` always expects this member to be called `__bool__`,
    even on Python 2.7 where the Python spelling was `__nonzero__`.

  * `def __call__(&self, parameter-list) -> PyResult<impl ToPyObject>`

    For details on `parameter-list`, see the documentation of `py_argparse!()`.
    The return type must be `PyResult<T>` for some `T` that implements `ToPyObject`.


*/
#[macro_export]
macro_rules! py_class {
    (class $class:ident |$py: ident| { $( $body:tt )* }) => (
        py_class_impl! {
            { $( $body )* }
            $class $py
            /* info: */ {
                /* base_type: */ $crate::PyObject,
                /* size: */ <$crate::PyObject as $crate::py_class::BaseObject>::size(),
                /* class_visibility: */ {},
                /* gc: */ {
                    /* traverse_proc: */ None,
                    /* traverse_data: */ [ /*name*/ ]
                },
                /* data: */ [ /* { offset, name, type } */ ]
                // TODO: base type, documentation, ...
            }
            /* slots: */ {
                /* type_slots */  [ /* slot: expr, */ ]
                /* as_number */   [ /* slot: expr, */ ]
                /* as_sequence */ [ /* slot: expr, */ ]
                /* as_mapping */  [ /* slot: expr, */ ]
                /* setitem_delitem */ [
                    sdi_setitem: {},
                    sdi_delitem: {},
                ]
            }
            /* impls: */ { /* impl body */ }
            /* members: */ { /* ident = expr; */ }
        }
    );
    (pub class $class:ident |$py: ident| { $( $body:tt )* }) => (
        py_class_impl! {
            { $( $body )* }
            $class $py
            /* info: */ {
                /* base_type: */ $crate::PyObject,
                /* size: */ <$crate::PyObject as $crate::py_class::BaseObject>::size(),
                /* class_visibility: */ {pub},
                /* gc: */ {
                    /* traverse_proc: */ None,
                    /* traverse_data: */ [ /*name*/ ]
                },
                /* data: */ [ /* { offset, name, type } */ ]
                // TODO: base type, documentation, ...
            }
            /* slots: */ {
                /* type_slots */  [ /* slot: expr, */ ]
                /* as_number */   [ /* slot: expr, */ ]
                /* as_sequence */ [ /* slot: expr, */ ]
                /* as_mapping */  [ /* slot: expr, */ ]
                /* setitem_delitem */ [
                    sdi_setitem: {},
                    sdi_delitem: {},
                ]
            }
            /* impls: */ { /* impl body */ }
            /* members: */ { /* ident = expr; */ }
        }
    );
}

#[macro_export]
#[doc(hidden)]
macro_rules! py_class_impl_item {
    { $class:ident, $py:ident, $name:ident( $( $selfarg:tt )* )
        $res_type:ty; $body:block [ $( { $pname:ident : $ptype:ty = $detail:tt } )* ]
    } => { py_coerce_item! {
        impl $class {
            pub fn $name($( $selfarg )* $py: $crate::Python $( , $pname: $ptype )* )
            -> $res_type $body
        }
    }}
}
