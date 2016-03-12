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
`py_class!(class MyType |py| { ... })`

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
`type` must be `Send + 'static`.

Because Python object instances can be freely shared (Python has no concept of "ownership"),
data fields cannot be declared as `mut`.
If mutability is required, you have to use interior mutability (`Cell` or `RefCell`).

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

## __new__
`def __new__(cls, parameter-list) -> PyResult<...> { ... }`

Declares a constructor method callable from Python.

* The first parameter is the type object of the class to create.
  This may be the type object of a derived class declared in Python.
* If no `__new__` method is declared, object instances can only be created from Rust (via `MyType::create_instance`),
  but not from Python.
* For details on `parameter-list`, see the documentation of `py_argparse!()`.
* The return type must be `PyResult<T>` for some `T` that implements `ToPyObject`.
  Usually, `T` will be `MyType`.

*/
#[macro_export]
macro_rules! py_class {
    (class $class:ident |$py: ident| { $( $body:tt )* }) => (
        py_class_impl! {
            $class $py
            /* info: */ {
                /* base_type: */ $crate::PyObject,
                /* size: */ <$crate::PyObject as $crate::py_class::BaseObject>::size(),
                /* data: */ [ /* { offset, name, type } */ ]
                // TODO: base type, documentation, ...
            }
            /* slots: */ {
                /* type_slots */  [ /* slot: expr, */ ]
                /* as_number */   [ /* slot: expr, */ ]
                /* as_sequence */ [ /* slot: expr, */ ]
            }
            /* impls: */ { /* impl body */ }
            /* members: */ { /* ident = expr; */ };
            $( $body )*
        }
    );
}

#[macro_export]
#[doc(hidden)]
macro_rules! py_class_impl {
    // TT muncher macro. Results are accumulated in $info $slots $impls and $members.
    
    // Base case: we're done munching and can start producing code:
    { $class:ident $py:ident
        /* info: */ {
            $base_type:ty,
            $size:expr,
            /* data: */ [ $( { $data_offset:expr, $data_name:ident, $data_ty:ty } )* ]
        }
        $slots:tt { $( $imp:item )* } $members:tt;
    } => {
        struct $class { _unsafe_inner: $crate::PyObject }

        pyobject_to_pyobject!($class);

        impl $crate::PythonObject for $class {
            #[inline]
            fn as_object(&self) -> &$crate::PyObject {
                &self._unsafe_inner
            }

            #[inline]
            fn into_object(self) -> $crate::PyObject {
                self._unsafe_inner
            }

            /// Unchecked downcast from PyObject to Self.
            /// Undefined behavior if the input object does not have the expected type.
            #[inline]
            unsafe fn unchecked_downcast_from(obj: $crate::PyObject) -> Self {
                $class { _unsafe_inner: obj }
            }

            /// Unchecked downcast from PyObject to Self.
            /// Undefined behavior if the input object does not have the expected type.
            #[inline]
            unsafe fn unchecked_downcast_borrow_from<'a>(obj: &'a $crate::PyObject) -> &'a Self {
                ::std::mem::transmute(obj)
            }
        }

        impl $crate::PythonObjectWithCheckedDowncast for $class {
            #[inline]
            fn downcast_from<'p>(py: $crate::Python<'p>, obj: $crate::PyObject) -> Result<$class, $crate::PythonObjectDowncastError<'p>> {
                if py.get_type::<$class>().is_instance(py, &obj) {
                    Ok($class { _unsafe_inner: obj })
                } else {
                    Err($crate::PythonObjectDowncastError(py))
                }
            }

            #[inline]
            fn downcast_borrow_from<'a, 'p>(py: $crate::Python<'p>, obj: &'a $crate::PyObject) -> Result<&'a $class, $crate::PythonObjectDowncastError<'p>> {
                if py.get_type::<$class>().is_instance(py, obj) {
                    unsafe { Ok(::std::mem::transmute(obj)) }
                } else {
                    Err($crate::PythonObjectDowncastError(py))
                }
            }
        }

        py_coerce_item! {
            impl $crate::py_class::BaseObject for $class {
                type InitType = ( $( $data_ty, )* );

                #[inline]
                fn size() -> usize {
                    $size
                }

                unsafe fn alloc(
                    py: $crate::Python,
                    ty: &$crate::PyType,
                    ( $( $data_name, )* ): Self::InitType
                ) -> $crate::PyResult<$crate::PyObject>
                {
                    let obj = try!(<$base_type as $crate::py_class::BaseObject>::alloc(py, ty, ()));
                    $( $crate::py_class::data_init::<$data_ty>(py, &obj, $data_offset, $data_name); )*
                    Ok(obj)
                }

                unsafe fn dealloc(py: $crate::Python, obj: *mut $crate::_detail::ffi::PyObject) {
                    $( $crate::py_class::data_drop::<$data_ty>(py, obj, $data_offset); )*
                    <$base_type as $crate::py_class::BaseObject>::dealloc(py, obj)
                }
            }
        }
        $($imp)*
        py_coerce_item! {
            impl $class {
                fn create_instance(py: $crate::Python $( , $data_name : $data_ty )* ) -> $crate::PyResult<$class> {
                    let obj = try!(unsafe {
                        <$class as $crate::py_class::BaseObject>::alloc(
                            py, &py.get_type::<$class>(), ( $($data_name,)* )
                        )
                    });
                    return Ok($class { _unsafe_inner: obj });

                    // hide statics in create_instance to avoid name conflicts
                    static mut type_object : $crate::_detail::ffi::PyTypeObject
                        = py_class_type_object_static_init!($class, $slots);
                    static mut init_active: bool = false;

                    // trait implementations that need direct access to type_object
                    impl $crate::PythonObjectWithTypeObject for $class {
                        fn type_object(py: $crate::Python) -> $crate::PyType {
                            unsafe {
                                if $crate::py_class::is_ready(py, &type_object) {
                                    $crate::PyType::from_type_ptr(py, &mut type_object)
                                } else {
                                    // automatically initialize the class on-demand
                                    <$class as $crate::py_class::PythonObjectFromPyClassMacro>::initialize(py)
                                        .expect(concat!("An error occurred while initializing class ", stringify!($class)))
                                }
                            }
                        }
                    }

                    impl $crate::py_class::PythonObjectFromPyClassMacro for $class {
                        fn initialize(py: $crate::Python) -> $crate::PyResult<$crate::PyType> {
                            unsafe {
                                if $crate::py_class::is_ready(py, &type_object) {
                                    return Ok($crate::PyType::from_type_ptr(py, &mut type_object));
                                }
                                assert!(!init_active,
                                    concat!("Reentrancy detected: already initializing class ",
                                    stringify!($class)));
                                init_active = true;
                                let res = init(py);
                                init_active = false;
                                res
                            }
                        }
                    }

                    fn init($py: $crate::Python) -> $crate::PyResult<$crate::PyType> {
                        py_class_type_object_dynamic_init!($class, $py, type_object);
                        py_class_init_members!($class, $py, type_object, $members);
                        unsafe {
                            if $crate::_detail::ffi::PyType_Ready(&mut type_object) == 0 {
                                Ok($crate::PyType::from_type_ptr($py, &mut type_object))
                            } else {
                                Err($crate::PyErr::fetch($py))
                            }
                        }
                    }
                }
            }
        }
    };

    // Data declaration
    { $class:ident $py:ident
        /* info: */ {
            $base_type: ty,
            $size: expr,
            [ $( $data:tt )* ]
        }
        $slots:tt { $( $imp:item )* } $members:tt;
        data $data_name:ident : $data_type:ty; $($tail:tt)*
    } => { py_class_impl! {
        $class $py
        /* info: */ {
            $base_type,
            /* size: */ $crate::py_class::data_new_size::<$data_type>($size),
            /* data: */ [
                $($data)*
                {
                    $crate::py_class::data_offset::<$data_type>($size),
                    $data_name,
                    $data_type
                }
            ]
        }
        $slots
        /* impl: */ {
            $($imp)*
            impl $class {
                fn $data_name<'a>(&'a self, py: $crate::Python<'a>) -> &'a $data_type {
                    unsafe {
                        $crate::py_class::data_get::<$data_type>(
                            py,
                            &self._unsafe_inner,
                            $crate::py_class::data_offset::<$data_type>($size)
                        )
                    }
                }
            }
        }
        $members;
        $($tail)*
    }};

    // def __new__(cls)
    { $class:ident $py:ident $info:tt
        /* slots: */ {
            /* type_slots */ [ $( $slot_name:ident : $slot_value:expr, )* ]
            $as_number:tt $as_sequence:tt
        }
        { $( $imp:item )* } $members:tt;
        def __new__ ($cls:ident)
            -> $res_type:ty { $( $body:tt )* } $($tail:tt)*
    } => { py_class_impl! {
        $class $py $info
        /* slots: */ {
            /* type_slots */ [
                $( $slot_name : $slot_value, )*
                tp_new: Some(py_class_wrap_newfunc!($class::__new__ [])),
            ]
            $as_number $as_sequence
        }
        /* impl: */ {
            $($imp)*
            py_class_impl_item! { $class, $py, __new__($cls: &$crate::PyType,) $res_type; { $($body)* } [] }
        }
        $members;
        $($tail)*
    }};
    // def __new__(cls, params)
    { $class:ident $py:ident $info:tt
        /* slots: */ {
            /* type_slots */ [ $( $slot_name:ident : $slot_value:expr, )* ]
            $as_number:tt $as_sequence:tt
        }
        { $( $imp:item )* } $members:tt;
        def __new__ ($cls:ident, $($p:tt)+)
            -> $res_type:ty { $( $body:tt )* } $($tail:tt)*
    } => { py_class_impl! {
        $class $py $info
        /* slots: */ {
            /* type_slots */ [
                $( $slot_name : $slot_value, )*
                tp_new: Some(py_argparse_parse_plist_impl!{
                    py_class_wrap_newfunc {$class::__new__}
                    [] ($($p)+,)
                }),
            ]
            $as_number $as_sequence
        }
        /* impl: */ {
            $($imp)*
            py_argparse_parse_plist_impl!{
                py_class_impl_item { $class, $py, __new__($cls: &$crate::PyType,) $res_type; { $($body)* } }
                [] ($($p)+,)
            }
        }
        $members;
        $($tail)*
    }};

    // def __init__()
    { $class:ident $py:ident $info:tt $slots:tt $impls:tt $members:tt;
        def __init__ $($tail:tt)*
    } => {
        py_error! { "__init__ is not supported by py_class!; use __new__ instead." }
    };

    // def __del__()
    { $class:ident $py:ident $info:tt $slots:tt $impls:tt $members:tt;
        def __del__ $($tail:tt)*
    } => {
        py_error! { "__del__ is not supported by py_class!; Use a data member with a Drop impl instead." }
    };

    // TODO: Not yet implemented:
    { $class:ident $py:ident $info:tt $slots:tt $impls:tt $members:tt;
        def __repr__ $($tail:tt)*
    } => {
        py_error! { "__repr__ is not supported by py_class! yet." }
    };

    { $class:ident $py:ident $info:tt $slots:tt $impls:tt $members:tt;
        def __str__ $($tail:tt)*
    } => {
        py_error! { "__str__ is not supported by py_class! yet." }
    };
    { $class:ident $py:ident $info:tt $slots:tt $impls:tt $members:tt;
        def __unicode__ $($tail:tt)*
    } => {
        py_error! { "__unicode__ is not supported by py_class! yet." }
    };
    { $class:ident $py:ident $info:tt $slots:tt $impls:tt $members:tt;
        def __bytes__ $($tail:tt)*
    } => {
        py_error! { "__bytes__ is not supported by py_class! yet." }
    };

    { $class:ident $py:ident $info:tt $slots:tt $impls:tt $members:tt;
        def __format__ $($tail:tt)*
    } => {
        py_error! { "__format__ is not supported by py_class! yet." }
    };

    // Comparison operators:
    { $class:ident $py:ident $info:tt $slots:tt $impls:tt $members:tt;
        def __lt__ $($tail:tt)*
    } => {
        py_error! { "__lt__ is not supported by py_class! yet." }
    };
    { $class:ident $py:ident $info:tt $slots:tt $impls:tt $members:tt;
        def __le__ $($tail:tt)*
    } => {
        py_error! { "__le__ is not supported by py_class! yet." }
    };
    { $class:ident $py:ident $info:tt $slots:tt $impls:tt $members:tt;
        def __eq__ $($tail:tt)*
    } => {
        py_error! { "__eq__ is not supported by py_class! yet." }
    };
    { $class:ident $py:ident $info:tt $slots:tt $impls:tt $members:tt;
        def __ne__ $($tail:tt)*
    } => {
        py_error! { "__ne__ is not supported by py_class! yet." }
    };
    { $class:ident $py:ident $info:tt $slots:tt $impls:tt $members:tt;
        def __gt__ $($tail:tt)*
    } => {
        py_error! { "__gt__ is not supported by py_class! yet." }
    };
    { $class:ident $py:ident $info:tt $slots:tt $impls:tt $members:tt;
        def __ge__ $($tail:tt)*
    } => {
        py_error! { "__ge__ is not supported by py_class! yet." }
    };
    { $class:ident $py:ident $info:tt $slots:tt $impls:tt $members:tt;
        def __cmp__ $($tail:tt)*
    } => {
        py_error! { "__cmp__ is not supported by py_class!." }
    };
    { $class:ident $py:ident $info:tt $slots:tt $impls:tt $members:tt;
        def __hash__ $($tail:tt)*
    } => {
        py_error! { "__hash__ is not supported by py_class! yet." }
    };
    { $class:ident $py:ident $info:tt $slots:tt $impls:tt $members:tt;
        def __nonzero__ $($tail:tt)*
    } => {
        py_error! { "__nonzero__ is not supported by py_class!; use the Python 3 spelling __bool__ instead." }
    };
    { $class:ident $py:ident $info:tt $slots:tt $impls:tt $members:tt;
        def __bool__ $($tail:tt)*
    } => {
        py_error! { "__bool__ is not supported by py_class! yet." }
    };
    
    // Customizing attribute access
    { $class:ident $py:ident $info:tt $slots:tt $impls:tt $members:tt;
        def __getattr__ $($tail:tt)*
    } => {
        py_error! { "__getattr__ is not supported by py_class! yet." }
    };
    { $class:ident $py:ident $info:tt $slots:tt $impls:tt $members:tt;
        def __getattribute__ $($tail:tt)*
    } => {
        py_error! { "__getattribute__ is not supported by py_class! yet." }
    };
    { $class:ident $py:ident $info:tt $slots:tt $impls:tt $members:tt;
        def __setattr__ $($tail:tt)*
    } => {
        py_error! { "__setattr__ is not supported by py_class! yet." }
    };
    { $class:ident $py:ident $info:tt $slots:tt $impls:tt $members:tt;
        def __delattr__ $($tail:tt)*
    } => {
        py_error! { "__delattr__ is not supported by py_class! yet." }
    };
    { $class:ident $py:ident $info:tt $slots:tt $impls:tt $members:tt;
        def __dir__ $($tail:tt)*
    } => {
        py_error! { "__dir__ is not supported by py_class! yet." }
    };
    
    // Implementing Descriptors
    { $class:ident $py:ident $info:tt $slots:tt $impls:tt $members:tt;
        def __get__ $($tail:tt)*
    } => {
        py_error! { "__get__ is not supported by py_class! yet." }
    };
    { $class:ident $py:ident $info:tt $slots:tt $impls:tt $members:tt;
        def __set__ $($tail:tt)*
    } => {
        py_error! { "__set__ is not supported by py_class! yet." }
    };
    { $class:ident $py:ident $info:tt $slots:tt $impls:tt $members:tt;
        def __delete__ $($tail:tt)*
    } => {
        py_error! { "__delete__ is not supported by py_class! yet." }
    };

    // Customizing instance and subclass checks
    { $class:ident $py:ident $info:tt $slots:tt $impls:tt $members:tt;
        def __instancecheck__ $($tail:tt)*
    } => {
        py_error! { "__instancecheck__ is not supported by py_class! yet." }
    };
    { $class:ident $py:ident $info:tt $slots:tt $impls:tt $members:tt;
        def __subclasscheck__ $($tail:tt)*
    } => {
        py_error! { "__subclasscheck__ is not supported by py_class! yet." }
    };

    // Emulating callable objects
    { $class:ident $py:ident $info:tt $slots:tt $impls:tt $members:tt;
        def __call__ $($tail:tt)*
    } => {
        py_error! { "__call__ is not supported by py_class! yet." }
    };
    
    // Emulating container types
    { $class:ident $py:ident $info:tt $slots:tt $impls:tt $members:tt;
        def __len__ $($tail:tt)*
    } => {
        py_error! { "__len__ is not supported by py_class! yet." }
    };
    { $class:ident $py:ident $info:tt $slots:tt $impls:tt $members:tt;
        def __length_hint__ $($tail:tt)*
    } => {
        py_error! { "__length_hint__ is not supported by py_class! yet." }
    };
    { $class:ident $py:ident $info:tt $slots:tt $impls:tt $members:tt;
        def __getitem__ $($tail:tt)*
    } => {
        py_error! { "__getitem__ is not supported by py_class! yet." }
    };
    { $class:ident $py:ident $info:tt $slots:tt $impls:tt $members:tt;
        def __missing__ $($tail:tt)*
    } => {
        py_error! { "__missing__ is not supported by py_class! yet." }
    };
    { $class:ident $py:ident $info:tt $slots:tt $impls:tt $members:tt;
        def __setitem__ $($tail:tt)*
    } => {
        py_error! { "__setitem__ is not supported by py_class! yet." }
    };
    { $class:ident $py:ident $info:tt $slots:tt $impls:tt $members:tt;
        def __delitem__ $($tail:tt)*
    } => {
        py_error! { "__delitem__ is not supported by py_class! yet." }
    };
    { $class:ident $py:ident $info:tt $slots:tt $impls:tt $members:tt;
        def __iter__ $($tail:tt)*
    } => {
        py_error! { "__iter__ is not supported by py_class! yet." }
    };
    { $class:ident $py:ident $info:tt $slots:tt $impls:tt $members:tt;
        def __next__ $($tail:tt)*
    } => {
        py_error! { "__next__ is not supported by py_class! yet." }
    };
    { $class:ident $py:ident $info:tt $slots:tt $impls:tt $members:tt;
        def __reversed__ $($tail:tt)*
    } => {
        py_error! { "__reversed__ is not supported by py_class! yet." }
    };
    { $class:ident $py:ident $info:tt $slots:tt $impls:tt $members:tt;
        def __contains__ $($tail:tt)*
    } => {
        py_error! { "__contains__ is not supported by py_class! yet." }
    };
    
    // Emulating numeric types
    { $class:ident $py:ident $info:tt $slots:tt $impls:tt $members:tt;
        def __add__ $($tail:tt)*
    } => {
        py_error! { "__add__ is not supported by py_class! yet." }
    };
    { $class:ident $py:ident $info:tt $slots:tt $impls:tt $members:tt;
        def __sub__ $($tail:tt)*
    } => {
        py_error! { "__sub__ is not supported by py_class! yet." }
    };
    { $class:ident $py:ident $info:tt $slots:tt $impls:tt $members:tt;
        def __mul__ $($tail:tt)*
    } => {
        py_error! { "__mul__ is not supported by py_class! yet." }
    };
    { $class:ident $py:ident $info:tt $slots:tt $impls:tt $members:tt;
        def __matmul__ $($tail:tt)*
    } => {
        py_error! { "__matmul__ is not supported by py_class! yet." }
    };
    { $class:ident $py:ident $info:tt $slots:tt $impls:tt $members:tt;
        def __div__ $($tail:tt)*
    } => {
        py_error! { "__div__ is not supported by py_class! yet." }
    };
    { $class:ident $py:ident $info:tt $slots:tt $impls:tt $members:tt;
        def __truediv__ $($tail:tt)*
    } => {
        py_error! { "__truediv__ is not supported by py_class! yet." }
    };
    { $class:ident $py:ident $info:tt $slots:tt $impls:tt $members:tt;
        def __floordiv__ $($tail:tt)*
    } => {
        py_error! { "__floordiv__ is not supported by py_class! yet." }
    };
    { $class:ident $py:ident $info:tt $slots:tt $impls:tt $members:tt;
        def __mod__ $($tail:tt)*
    } => {
        py_error! { "__mod__ is not supported by py_class! yet." }
    };
    { $class:ident $py:ident $info:tt $slots:tt $impls:tt $members:tt;
        def __divmod__ $($tail:tt)*
    } => {
        py_error! { "__divmod__ is not supported by py_class! yet." }
    };
    { $class:ident $py:ident $info:tt $slots:tt $impls:tt $members:tt;
        def __pow__ $($tail:tt)*
    } => {
        py_error! { "__pow__ is not supported by py_class! yet." }
    };
    { $class:ident $py:ident $info:tt $slots:tt $impls:tt $members:tt;
        def __lshift__ $($tail:tt)*
    } => {
        py_error! { "__lshift__ is not supported by py_class! yet." }
    };
    { $class:ident $py:ident $info:tt $slots:tt $impls:tt $members:tt;
        def __rshift__ $($tail:tt)*
    } => {
        py_error! { "__rshift__ is not supported by py_class! yet." }
    };
    { $class:ident $py:ident $info:tt $slots:tt $impls:tt $members:tt;
        def __and__ $($tail:tt)*
    } => {
        py_error! { "__and__ is not supported by py_class! yet." }
    };
    { $class:ident $py:ident $info:tt $slots:tt $impls:tt $members:tt;
        def __xor__ $($tail:tt)*
    } => {
        py_error! { "__xor__ is not supported by py_class! yet." }
    };
    { $class:ident $py:ident $info:tt $slots:tt $impls:tt $members:tt;
        def __or__ $($tail:tt)*
    } => {
        py_error! { "__or__ is not supported by py_class! yet." }
    };
    
    // Emulating numeric types - reflected
    { $class:ident $py:ident $info:tt $slots:tt $impls:tt $members:tt;
        def __radd__ $($tail:tt)*
    } => {
        py_error! { "__radd__ is not supported by py_class! yet." }
    };
    { $class:ident $py:ident $info:tt $slots:tt $impls:tt $members:tt;
        def __rsub__ $($tail:tt)*
    } => {
        py_error! { "__rsub__ is not supported by py_class! yet." }
    };
    { $class:ident $py:ident $info:tt $slots:tt $impls:tt $members:tt;
        def __rmul__ $($tail:tt)*
    } => {
        py_error! { "__rmul__ is not supported by py_class! yet." }
    };
    { $class:ident $py:ident $info:tt $slots:tt $impls:tt $members:tt;
        def __rmatmul__ $($tail:tt)*
    } => {
        py_error! { "__rmatmul__ is not supported by py_class! yet." }
    };
    { $class:ident $py:ident $info:tt $slots:tt $impls:tt $members:tt;
        def __rdiv__ $($tail:tt)*
    } => {
        py_error! { "__rdiv__ is not supported by py_class! yet." }
    };
    { $class:ident $py:ident $info:tt $slots:tt $impls:tt $members:tt;
        def __rtruediv__ $($tail:tt)*
    } => {
        py_error! { "__rtruediv__ is not supported by py_class! yet." }
    };
    { $class:ident $py:ident $info:tt $slots:tt $impls:tt $members:tt;
        def __rfloordiv__ $($tail:tt)*
    } => {
        py_error! { "__rfloordiv__ is not supported by py_class! yet." }
    };
    { $class:ident $py:ident $info:tt $slots:tt $impls:tt $members:tt;
        def __rmod__ $($tail:tt)*
    } => {
        py_error! { "__rmod__ is not supported by py_class! yet." }
    };
    { $class:ident $py:ident $info:tt $slots:tt $impls:tt $members:tt;
        def __rdivmod__ $($tail:tt)*
    } => {
        py_error! { "__rdivmod__ is not supported by py_class! yet." }
    };
    { $class:ident $py:ident $info:tt $slots:tt $impls:tt $members:tt;
        def __rpow__ $($tail:tt)*
    } => {
        py_error! { "__rpow__ is not supported by py_class! yet." }
    };
    { $class:ident $py:ident $info:tt $slots:tt $impls:tt $members:tt;
        def __rlshift__ $($tail:tt)*
    } => {
        py_error! { "__rlshift__ is not supported by py_class! yet." }
    };
    { $class:ident $py:ident $info:tt $slots:tt $impls:tt $members:tt;
        def __rrshift__ $($tail:tt)*
    } => {
        py_error! { "__rrshift__ is not supported by py_class! yet." }
    };
    { $class:ident $py:ident $info:tt $slots:tt $impls:tt $members:tt;
        def __rand__ $($tail:tt)*
    } => {
        py_error! { "__rand__ is not supported by py_class! yet." }
    };
    { $class:ident $py:ident $info:tt $slots:tt $impls:tt $members:tt;
        def __rxor__ $($tail:tt)*
    } => {
        py_error! { "__rxor__ is not supported by py_class! yet." }
    };
    { $class:ident $py:ident $info:tt $slots:tt $impls:tt $members:tt;
        def __ror__ $($tail:tt)*
    } => {
        py_error! { "__ror__ is not supported by py_class! yet." }
    };

    // Emulating numeric types - in-place
    { $class:ident $py:ident $info:tt $slots:tt $impls:tt $members:tt;
        def __iadd__ $($tail:tt)*
    } => {
        py_error! { "__iadd__ is not supported by py_class! yet." }
    };
    { $class:ident $py:ident $info:tt $slots:tt $impls:tt $members:tt;
        def __isub__ $($tail:tt)*
    } => {
        py_error! { "__isub__ is not supported by py_class! yet." }
    };
    { $class:ident $py:ident $info:tt $slots:tt $impls:tt $members:tt;
        def __imul__ $($tail:tt)*
    } => {
        py_error! { "__imul__ is not supported by py_class! yet." }
    };
    { $class:ident $py:ident $info:tt $slots:tt $impls:tt $members:tt;
        def __imatmul__ $($tail:tt)*
    } => {
        py_error! { "__imatmul__ is not supported by py_class! yet." }
    };
    { $class:ident $py:ident $info:tt $slots:tt $impls:tt $members:tt;
        def __idiv__ $($tail:tt)*
    } => {
        py_error! { "__idiv__ is not supported by py_class! yet." }
    };
    { $class:ident $py:ident $info:tt $slots:tt $impls:tt $members:tt;
        def __itruediv__ $($tail:tt)*
    } => {
        py_error! { "__itruediv__ is not supported by py_class! yet." }
    };
    { $class:ident $py:ident $info:tt $slots:tt $impls:tt $members:tt;
        def __ifloordiv__ $($tail:tt)*
    } => {
        py_error! { "__ifloordiv__ is not supported by py_class! yet." }
    };
    { $class:ident $py:ident $info:tt $slots:tt $impls:tt $members:tt;
        def __imod__ $($tail:tt)*
    } => {
        py_error! { "__imod__ is not supported by py_class! yet." }
    };
    { $class:ident $py:ident $info:tt $slots:tt $impls:tt $members:tt;
        def __idivmod__ $($tail:tt)*
    } => {
        py_error! { "__idivmod__ is not supported by py_class! yet." }
    };
    { $class:ident $py:ident $info:tt $slots:tt $impls:tt $members:tt;
        def __ipow__ $($tail:tt)*
    } => {
        py_error! { "__ipow__ is not supported by py_class! yet." }
    };
    { $class:ident $py:ident $info:tt $slots:tt $impls:tt $members:tt;
        def __ilshift__ $($tail:tt)*
    } => {
        py_error! { "__ilshift__ is not supported by py_class! yet." }
    };
    { $class:ident $py:ident $info:tt $slots:tt $impls:tt $members:tt;
        def __irshift__ $($tail:tt)*
    } => {
        py_error! { "__irshift__ is not supported by py_class! yet." }
    };
    { $class:ident $py:ident $info:tt $slots:tt $impls:tt $members:tt;
        def __iand__ $($tail:tt)*
    } => {
        py_error! { "__iand__ is not supported by py_class! yet." }
    };
    { $class:ident $py:ident $info:tt $slots:tt $impls:tt $members:tt;
        def __ixor__ $($tail:tt)*
    } => {
        py_error! { "__ixor__ is not supported by py_class! yet." }
    };
    { $class:ident $py:ident $info:tt $slots:tt $impls:tt $members:tt;
        def __ior__ $($tail:tt)*
    } => {
        py_error! { "__ior__ is not supported by py_class! yet." }
    };

    // Unary arithmetic
    { $class:ident $py:ident $info:tt $slots:tt $impls:tt $members:tt;
        def __neg__ $($tail:tt)*
    } => {
        py_error! { "__neg__ is not supported by py_class! yet." }
    };
    { $class:ident $py:ident $info:tt $slots:tt $impls:tt $members:tt;
        def __pos__ $($tail:tt)*
    } => {
        py_error! { "__pos__ is not supported by py_class! yet." }
    };
    { $class:ident $py:ident $info:tt $slots:tt $impls:tt $members:tt;
        def __abs__ $($tail:tt)*
    } => {
        py_error! { "__abs__ is not supported by py_class! yet." }
    };
    { $class:ident $py:ident $info:tt $slots:tt $impls:tt $members:tt;
        def __invert__ $($tail:tt)*
    } => {
        py_error! { "__invert__ is not supported by py_class! yet." }
    };
    { $class:ident $py:ident $info:tt $slots:tt $impls:tt $members:tt;
        def __complex__ $($tail:tt)*
    } => {
        py_error! { "__complex__ is not supported by py_class! yet." }
    };
    { $class:ident $py:ident $info:tt $slots:tt $impls:tt $members:tt;
        def __int__ $($tail:tt)*
    } => {
        py_error! { "__int__ is not supported by py_class! yet." }
    };
    { $class:ident $py:ident $info:tt $slots:tt $impls:tt $members:tt;
        def __long__ $($tail:tt)*
    } => {
        py_error! { "__long__ is not supported by py_class! yet." }
    };
    { $class:ident $py:ident $info:tt $slots:tt $impls:tt $members:tt;
        def __float__ $($tail:tt)*
    } => {
        py_error! { "__float__ is not supported by py_class! yet." }
    };
    { $class:ident $py:ident $info:tt $slots:tt $impls:tt $members:tt;
        def __round__ $($tail:tt)*
    } => {
        py_error! { "__round__ is not supported by py_class! yet." }
    };
    { $class:ident $py:ident $info:tt $slots:tt $impls:tt $members:tt;
        def __index__ $($tail:tt)*
    } => {
        py_error! { "__index__ is not supported by py_class! yet." }
    };
    { $class:ident $py:ident $info:tt $slots:tt $impls:tt $members:tt;
        def __coerce__ $($tail:tt)*
    } => {
        py_error! { "__coerce__ is not supported by py_class! yet." }
    };

    // With statement context managers
    { $class:ident $py:ident $info:tt $slots:tt $impls:tt $members:tt;
        def __enter__ $($tail:tt)*
    } => {
        py_error! { "__enter__ is not supported by py_class! yet." }
    };
    { $class:ident $py:ident $info:tt $slots:tt $impls:tt $members:tt;
        def __exit__ $($tail:tt)*
    } => {
        py_error! { "__exit__ is not supported by py_class! yet." }
    };

    // Coroutines
    { $class:ident $py:ident $info:tt $slots:tt $impls:tt $members:tt;
        def __await__ $($tail:tt)*
    } => {
        py_error! { "__await__ is not supported by py_class! yet." }
    };
    { $class:ident $py:ident $info:tt $slots:tt $impls:tt $members:tt;
        def __aiter__ $($tail:tt)*
    } => {
        py_error! { "__aiter__ is not supported by py_class! yet." }
    };
    { $class:ident $py:ident $info:tt $slots:tt $impls:tt $members:tt;
        def __aenter__ $($tail:tt)*
    } => {
        py_error! { "__aenter__ is not supported by py_class! yet." }
    };
    { $class:ident $py:ident $info:tt $slots:tt $impls:tt $members:tt;
        def __aexit__ $($tail:tt)*
    } => {
        py_error! { "__aexit__ is not supported by py_class! yet." }
    };

    // def instance_method(&self)
    { $class:ident $py:ident $info:tt $slots:tt
        { $( $imp:item )* }
        { $( $member_name:ident = $member_expr:expr; )* };
        def $name:ident (&$slf:ident)
            -> $res_type:ty { $( $body:tt )* } $($tail:tt)*
    } => { py_class_impl! {
        $class $py $info $slots
        /* impl: */ {
            $($imp)*
            py_class_impl_item! { $class, $py, $name(&$slf,) $res_type; { $($body)* } [] }
        }
        /* members: */ {
            $( $member_name:ident = $member_expr:expr; )*
            $name = py_class_instance_method!{$py, $class::$name []};
        };
        $($tail)*
    }};
    // def instance_method(&self, params)
    { $class:ident $py:ident $info:tt $slots:tt
        { $( $imp:item )* }
        { $( $member_name:ident = $member_expr:expr; )* };
        def $name:ident (&$slf:ident, $($p:tt)+)
            -> $res_type:ty { $( $body:tt )* } $($tail:tt)*
    } => { py_class_impl! {
        $class $py $info $slots
        /* impl: */ {
            $($imp)*
            py_argparse_parse_plist_impl!{
                py_class_impl_item { $class, $py, $name(&$slf,) $res_type; { $($body)* } }
                [] ($($p)+,)
            }
        }
        /* members: */ {
            $( $member_name:ident = $member_expr:expr; )*
            $name = py_argparse_parse_plist_impl!{
                py_class_instance_method!{$py, $class::$name}
                [] ($($p)+,)
            };
        };
        $($tail)*
    }};
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

