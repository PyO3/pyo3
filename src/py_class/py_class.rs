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
# Example
```
#[macro_use] extern crate cpython;
use cpython::{Python, PyResult, PyType, PyDict};

py_class!(class MyType |py| {
    data number: i32;
/*
    def __new__(_cls: &PyType, arg: i32) -> PyResult<MyType> {
        Ok(MyType::create_instance(py, arg))
    }
    def half(&self) -> PyResult<i32> {
        println!("half() was called with self={:?}", self.number(py));
        Ok(self.number(py) / 2)
    }*/
});

fn main() {
    let gil = Python::acquire_gil();
    let py = gil.python();
    let dict = PyDict::new(py);
    dict.set_item(py, "MyType", py.get_type::<MyType>()).unwrap();
    py.run("assert MyType(42).half() == 21", None, Some(&dict)).unwrap();
}
``` */
#[macro_export]
macro_rules! py_class {
    (class $name:ident |$py: ident| { $( $body:tt )* }) => (
        py_class_impl! {
            $name $py
            /* info: */ {
                /* size: */ ::std::mem::size_of::<$crate::PyObject>(),
                /* data: */ [ /* { offset, name, type } */ ]
                // TODO: base type, documentation, ...
            }
            /* slots: */ {
                /* type_slots */  [ /* slot: expr, */ ]
                /* as_number */   [ /* slot: expr, */ ]
                /* as_sequence */ [ /* slot: expr, */ ]
            }
            /* impls: */ { /* impl body */ }
            /* members: */ { /* ident: expr, */ };
            $( $body )*
        }
    );
}

#[macro_export]
#[doc(hidden)]
macro_rules! py_class_impl {
    // TT muncher macro. Results are accumulated in $info $slots $impls and $members.
    
    // Base case: we're done munching and can start producing code:
    { $name:ident $py:ident
        /* info: */ {
            $size: expr,
            /* data: */ [ $( { $data_offset:expr, $data_name:ident, $data_ty:ty } )* ]
        }
        $slots:tt { $( $impl_tt:tt )* } $members:tt;
    } => {
        struct $name($crate::PyObject);
        pyobject_newtype!($name, downcast using typeobject);
        py_coerce_item! {
            impl $name {
                $($impl_tt)*

                fn create_instance(py: $crate::Python $( , $data_name : $data_ty )* ) -> $name {
                    unimplemented!();

                    // hide statics in create_instance to avoid name conflicts
                    static mut type_object : $crate::_detail::ffi::PyTypeObject
                        = py_class_type_object_static_init!($name, $size, $slots);
                    static mut init_active: bool = false;

                    // trait implementations that need direct access to type_object
                    impl $crate::PythonObjectWithTypeObject for $name {
                        fn type_object(py: $crate::Python) -> $crate::PyType {
                            unsafe {
                                if $crate::py_class::is_ready(py, &type_object) {
                                    $crate::PyType::from_type_ptr(py, &mut type_object)
                                } else {
                                    // automatically initialize the class on-demand
                                    <$name as $crate::py_class::PythonObjectFromPyClassMacro>::initialize(py)
                                        .expect("An error occurred while initializing class ")
                                }
                            }
                        }
                    }

                    impl $crate::py_class::PythonObjectFromPyClassMacro for $name {
                        fn initialize(py: Python) -> $crate::PyResult<$crate::PyType> {
                            unsafe {
                                if $crate::py_class::is_ready(py, &type_object) {
                                    return Ok($crate::PyType::from_type_ptr(py, &mut type_object));
                                }
                                assert!(!init_active,
                                    concat!("Reentrancy detected: already initializing class ",
                                    stringify!($name)));
                                init_active = true;
                                let res = init(py);
                                init_active = false;
                                res
                            }
                        }
                    }

                    unsafe fn init(py: Python) -> $crate::PyResult<$crate::PyType> {
                        py_class_type_object_dynamic_init!(py, type_object, $name, $size);
                        if $crate::_detail::ffi::PyType_Ready(&mut type_object) == 0 {
                            Ok($crate::PyType::from_type_ptr(py, &mut type_object))
                        } else {
                            Err($crate::PyErr::fetch(py))
                        }
                    }
                }
            }
        }
    };

    // Data declaration
    { $name:ident $py:ident
        /* info: */ {
            $size: expr,
            [ $( $data:tt )* ]
        }
        $slots:tt { $( $impl_tt:tt )* } $members:tt;
        data $data_name:ident : $data_type:ty; $($tail:tt)*
    } => { py_class_impl! {
        $name $py
        /* info: */ {
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
            $($impl_tt)*
            pub fn $data_name<'a>(&'a self, py: $crate::Python<'a>) -> &'a $data_type {
                unsafe {
                    $crate::py_class::data_get::<$data_type>(
                        py,
                        &self.0,
                        $crate::py_class::data_offset::<$data_type>($size)
                    )
                }
            }
        }
        $members;
        $($tail)*
    }}
}

