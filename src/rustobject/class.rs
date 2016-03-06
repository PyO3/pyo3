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

use python::{self, Python};
use objects::PyType;
use err::PyResult;
use ffi;

/// Trait implemented by the types produced by the `py_class!()` macro.
pub trait PythonObjectFromPyClassMacro : python::PythonObjectWithTypeObject {
    fn initialize(py: Python) -> PyResult<PyType>;
}

/**
# Example
```
#[macro_use] extern crate cpython;
use cpython::{Python, PyResult, PyType, PyDict};

py_class!(class MyType, data: i32, |py| {
    def __new__(_cls: &PyType, arg: i32) -> PyResult<MyType> {
        Ok(MyType::create_instance(py, arg))
    }
    def half(&self) -> PyResult<i32> {
        println!("half() was called with self={:?}", self.data(py));
        Ok(self.data(py) / 2)
    }
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
    (pub class $name:ident, $data_name:ident : $data_ty:ty, |$py: ident| { $( $body:tt )* }) => (
        pub struct $name($crate::rustobject::PyRustObject<$data_ty>);
        py_class_impl!($name, $data_name: $data_ty,
            ($data_name: $data_ty),
            ($data_name, ()),
            |$py| { $( $body )* });
    );
    (pub class $name:ident($base:ty), $data_name:ident : $data_ty:ty, |$py: ident| { $( $body:tt )* }) => (
        pub struct $name($crate::rustobject::PyRustObject<$data_ty, $base>);
        py_class_impl!($name, $data_name: $data_ty,
            ($data_name: $data_ty, base_data: <$base as $crate::rustobject::BaseObject>::InitType),
            ($data_name, base_data),
            |$py| { $( $body )* });
    );
    (class $name:ident, $data_name:ident : $data_ty:ty, |$py: ident| { $( $body:tt )* }) => (
        struct $name($crate::rustobject::PyRustObject<$data_ty>);
        py_class_impl!($name, $data_name: $data_ty,
            ($data_name: $data_ty),
            ($data_name, ()),
            |$py| { $( $body )* });
    );
    (class $name:ident($base:ty), $data_name:ident : $data_ty:ty, |$py: ident| { $( $body:tt )* }) => (
        struct $name($crate::rustobject::PyRustObject<$data_ty, $base>);
        py_class_impl!($name, $data_name: $data_ty,
            ($data_name: $data_ty, base_data: <$base as $crate::rustobject::BaseObject>::InitType),
            ($data_name, base_data),
            |$py| { $( $body )* });
    );
}

#[macro_export]
#[doc(hidden)]
macro_rules! py_class_impl {
    (
        $name:ident,
        $data_name:ident : $data_ty:ty,
        ( $( $param_name:ident : $param_ty:ty ),* ),
        $init_val:expr,
        |$py: ident| { $( $body:tt )* }
    ) => (
        impl $name {
            pub fn $data_name<'a>(&'a self, py: $crate::Python<'a>) -> &'a $data_ty {
                self.0.get(py)
            }

            pub fn create_instance(py: $crate::Python, $( $param_name : $param_ty ),* ) -> $name {
                // hide statics in create_instance to avoid name conflicts
                static mut type_ptr: *mut $crate::_detail::ffi::PyTypeObject = 0 as *mut _;
                static mut init_active: bool = false;
                py_class_impl_py_object!($name, type_ptr);

                impl $crate::rustobject::PythonObjectFromPyClassMacro for $name {
                    fn initialize(py: Python) -> $crate::PyResult<$crate::PyType> {
                        unsafe {
                            if !type_ptr.is_null() {
                                return Ok(py.get_type::<$name>());
                            }
                            assert!(!init_active, "Reentrancy detected: already initializing class $name");
                            init_active = true;
                            let result = init(py);
                            if let Ok(ref ty) = result {
                                type_ptr = ty.as_type_ptr();
                                $crate::_detail::ffi::Py_INCREF(type_ptr as *mut $crate::_detail::ffi::PyObject);
                            }
                            init_active = false;
                            result
                        }
                    }
                }
                fn init($py: Python) -> $crate::PyResult<$crate::PyType> {
                    let mut b = $crate::rustobject::TypeBuilder::<$data_ty>::new(
                        $py, stringify!($name));
                    //let b = b.base(); TODO inheritance
                    py_class_parse_body!($name, $py, b, $( $body )* );
                    let ty = try!(b.finish());
                    Ok($crate::ToPyObject::into_py_object(ty, $py))
                }

                py_class_create_instance_impl!(py, $name, $init_val)
            }
        }
    );
}

#[macro_export]
#[doc(hidden)]
macro_rules! py_class_create_instance_impl {
    ($py: expr, $name: ident, $data: expr) => {{
        let type_obj = $py.get_type::<$name>();
        let obj = unsafe { $crate::rustobject::BaseObject::alloc($py, &type_obj, $data) };
        $crate::PyDrop::release_ref(type_obj, $py);
        $name(obj.expect("Allocation failed"))
    }}
}

#[macro_export]
#[doc(hidden)]
macro_rules! py_class_impl_py_object {
    ($name:ident, $type_obj:expr) => (
        impl $crate::PythonObject for $name {
            #[inline]
            fn as_object(&self) -> &$crate::PyObject {
                $crate::PythonObject::as_object(&self.0)
            }

            #[inline]
            fn into_object(self) -> $crate::PyObject {
                $crate::PythonObject::into_object(self.0)
            }

            /// Unchecked downcast from PyObject to Self.
            /// Undefined behavior if the input object does not have the expected type.
            #[inline]
            unsafe fn unchecked_downcast_from(obj: $crate::PyObject) -> Self {
                $name($crate::PythonObject::unchecked_downcast_from(obj))
            }

            /// Unchecked downcast from PyObject to Self.
            /// Undefined behavior if the input object does not have the expected type.
            #[inline]
            unsafe fn unchecked_downcast_borrow_from<'a>(obj: &'a $crate::PyObject) -> &'a Self {
                ::std::mem::transmute(obj)
            }
        }

        impl $crate::PythonObjectWithCheckedDowncast for $name {
            fn downcast_from<'p>(py: $crate::Python<'p>, obj: $crate::PyObject) -> Result<$name, $crate::PythonObjectDowncastError<'p>> {
                unsafe {
                    if $crate::_detail::ffi::PyObject_TypeCheck(obj.as_ptr(), type_ptr) != 0 {
                        Ok($name($crate::PythonObject::unchecked_downcast_from(obj)))
                    } else {
                        Err($crate::PythonObjectDowncastError(py))
                    }
                }
            }

            fn downcast_borrow_from<'a, 'p>(py: $crate::Python<'p>, obj: &'a $crate::PyObject) -> Result<&'a $name, $crate::PythonObjectDowncastError<'p>> {
                unsafe {
                    if $crate::_detail::ffi::PyObject_TypeCheck(obj.as_ptr(), type_ptr) != 0 {
                        Ok(::std::mem::transmute(obj))
                    } else {
                        Err($crate::PythonObjectDowncastError(py))
                    }
                }
            }
        }

        impl $crate::PythonObjectWithTypeObject for $name {
            fn type_object(py: $crate::Python) -> $crate::PyType {
                unsafe {
                    if type_ptr.is_null() {
                        // automatically initialize the class on-demand
                        <$name as $crate::rustobject::PythonObjectFromPyClassMacro>::initialize(py).unwrap()
                    } else {
                        $crate::PyType::from_type_ptr(py, type_ptr)
                    }
                }
            }
        }

        impl $crate::ToPyObject for $name {
            type ObjectType = $name;

            #[inline]
            fn to_py_object(&self, py: $crate::Python) -> $name {
                $crate::PyClone::clone_ref(self, py)
            }

            #[inline]
            fn into_py_object(self, _py: $crate::Python) -> $name {
                self
            }

            #[inline]
            fn with_borrowed_ptr<F, R>(&self, _py: $crate::Python, f: F) -> R
                where F: FnOnce(*mut $crate::_detail::ffi::PyObject) -> R
            {
                f($crate::PythonObject::as_object(&self.0).as_ptr())
            }
        }
    )
}

#[macro_export]
#[doc(hidden)]
macro_rules! py_class_parse_body {
    ($class: ident, $py: ident, $b: ident, ) => ( );
    ($class: ident, $py: ident, $b: ident,
        def __new__ ($cls:ident : $cls_type:ty, $( $plist:tt )*)
            -> $res_type:ty { $( $body:tt )* } $($remainder:tt)*
    ) => (
        py_class_new!($class, $py, $b, $cls:$cls_type,
            (,$($plist)*,) -> $res_type; { $($body)* });
        py_class_parse_body!($class, $py, $b, $($remainder)*);
    );
    ($class: ident, $py: ident, $b: ident,
        def $name:ident(&$slf:ident $( $plist:tt )*)
            -> $res_type:ty { $( $body:tt )* } $($remainder:tt)*
    ) => (
        py_argparse_declare_item_in_impl!{{impl $class} 
            {fn $name} (&$slf, $py: Python) ($($plist)*,) {
                -> $res_type {
                    py_coerce_expr!({$( $body )*})
                }
            }}
        {
            unsafe extern "C" fn wrap<DUMMY>(
                slf: *mut $crate::_detail::ffi::PyObject,
                args: *mut $crate::_detail::ffi::PyObject,
                kwargs: *mut $crate::_detail::ffi::PyObject)
            -> *mut $crate::_detail::ffi::PyObject
            {
                py_wrap_argparse!(py, concat!(stringify!($class), ".", stringify!($name), "()"), args, kwargs,
                    ( $($plist)*, ) {
                        let slf: $class = $crate::PyObject::from_borrowed_ptr(py, slf).unchecked_cast_into();
                        //let ret = $class::$name( &slf, py $(,$pname)* );
                        let ret = py_argparse_call_with_names!($class::$name, (&slf, py) ($($plist)*,));
                        $crate::PyDrop::release_ref(slf, py);
                        ret
                    })
            }
            unsafe {
                let method_def = py_method_def!($name, 0, wrap::<()>);
                $b.add(stringify!($name),
                    $crate::rustobject::py_method_impl::create_without_type_check(method_def));
            }
        }
        py_class_parse_body!($class, $py, $b, $($remainder)*);
    );
}

#[macro_export]
#[doc(hidden)]
macro_rules! py_class_new {
    ($class: ident, $py: ident, $b: ident, $cls:ident : $cls_type:ty,
        $plist: tt -> $res_type:ty; $body:block) => (
        py_argparse_declare_item_in_impl!{{impl $class}
            {fn __new__} ($py: Python, $cls: $cls_type) $plist { -> $res_type $body }
        }
        {
            unsafe extern "C" fn wrap_new(
                cls: *mut $crate::_detail::ffi::PyTypeObject,
                args: *mut $crate::_detail::ffi::PyObject,
                kwargs: *mut $crate::_detail::ffi::PyObject)
            -> *mut $crate::_detail::ffi::PyObject
            {
                py_wrap_argparse!(py, concat!(stringify!($class), ".__new__()"),
                    args, kwargs, $plist {
                        let cls = $crate::PyType::from_type_ptr(py, cls);
                        //let ret = $class::__new__( py, &cls, $($pname),* );
                        let ret = py_argparse_call_with_names!($class::__new__, (py, &cls) $plist);
                        $crate::PyDrop::release_ref(cls, py);
                        ret
                    })
            }
            $b.set_new(wrap_new as $crate::_detail::ffi::newfunc);
        }
    );
}

