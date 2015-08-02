// Copyright (c) 2015 Daniel Grunwald
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

use std::marker;
use python::{Python, PythonObject};
use objects::{PyObject, PyTuple, PyType};
use super::typebuilder::TypeMember;
use ffi;
use err;

/// Creates a Python instance method descriptor that invokes a Rust function.
///
/// As arguments, takes the name of a rust function with the signature
/// `for<'p> fn(&PyRustObject<'p, T>, &PyTuple<'p>) -> PyResult<'p, R>`
/// for some `R` that implements `ToPyObject`.
///
/// Returns a type that implements `typebuilder::TypeMember<PyRustObject<T>>`
/// by producing an instance method descriptor.
///
/// # Example
/// ```
/// #![feature(plugin)]
/// #![plugin(interpolate_idents)]
/// #[macro_use] extern crate cpython;
/// use cpython::{Python, PythonObject, PyResult, PyErr, ObjectProtocol,
///               PyTuple, PyRustObject, PyRustTypeBuilder};
/// use cpython::{exc};
///
/// fn mul<'p>(slf: &PyRustObject<'p, i32>, args: &PyTuple<'p>) -> PyResult<'p, i32> {
///     let py = slf.python();
///     match slf.get().checked_mul(try!(args.get_item(0).extract::<i32>())) {
///         Some(val) => Ok(val),
///         None => Err(PyErr::new_lazy_init(py.get_type::<exc::OverflowError>(), None))
///     }
/// }
///
/// fn main() {
///     let gil = Python::acquire_gil();
///     let multiplier_type = PyRustTypeBuilder::<i32>::new(gil.python(), "Multiplier")
///       .add("mul", py_method!(mul))
///       .finish().unwrap();
///     let obj = multiplier_type.create_instance(3, ()).into_object();
///     let result = obj.call_method("mul", &(4,), None).unwrap().extract::<i32>().unwrap();
///     assert_eq!(result, 12);
/// }
/// ```
#[macro_export]
macro_rules! py_method {
    ($f: ident) => ( interpolate_idents! {{
        unsafe extern "C" fn [ wrap_ $f ](
            slf: *mut $crate::_detail::ffi::PyObject,
            args: *mut $crate::_detail::ffi::PyObject)
        -> *mut $crate::_detail::ffi::PyObject
        {
            let _guard = $crate::_detail::PanicGuard::with_message("Rust panic in py_method!");
            let py = $crate::_detail::bounded_assume_gil_acquired(&args);
            let slf = $crate::PyObject::from_borrowed_ptr(py, slf);
            let slf = $crate::PythonObject::unchecked_downcast_from(slf);
            let args = $crate::PyObject::from_borrowed_ptr(py, args);
            let args = <$crate::PyTuple as $crate::PythonObject>::unchecked_downcast_from(args);
            match $f(&slf, &args) {
                Ok(val) => {
                    let obj = $crate::ToPyObject::into_py_object(val, py);
                    return $crate::PythonObject::into_object(obj).steal_ptr();
                }
                Err(e) => {
                    e.restore();
                    return ::std::ptr::null_mut();
                }
            }
        }
        static mut [ method_def_ $f ]: $crate::_detail::ffi::PyMethodDef = $crate::_detail::ffi::PyMethodDef {
            //ml_name: bytes!(stringify!($f), "\0"),
            ml_name: 0 as *const $crate::_detail::libc::c_char,
            ml_meth: Some([ wrap_ $f ]),
            ml_flags: $crate::_detail::ffi::METH_VARARGS,
            ml_doc: 0 as *const $crate::_detail::libc::c_char
        };
        unsafe {
            [ method_def_ $f ].ml_name =
                concat!(stringify!($f), "\0").as_ptr() as *const _;
            $crate::_detail::py_method_impl(&mut [ method_def_ $f ], $f)
        }
    }})
}

pub struct MethodDescriptor<T>(*mut ffi::PyMethodDef, marker::PhantomData<fn(&T)>);

// py_method_impl takes fn(&T) to ensure that the T in MethodDescriptor<T>
// corresponds to the T in the function signature.
pub unsafe fn py_method_impl<'p, T, R>(
    def: *mut ffi::PyMethodDef,
    _f: fn(&T, &PyTuple<'p>) -> err::PyResult<'p, R>
) -> MethodDescriptor<T> {
    MethodDescriptor(def, marker::PhantomData)
}

impl <'p, T> TypeMember<'p, T> for MethodDescriptor<T> where T: PythonObject<'p> {
    #[inline]
    fn to_descriptor(&self, ty: &PyType<'p>, _name: &str) -> PyObject<'p> {
        unsafe {
            err::from_owned_ptr_or_panic(ty.python(),
                ffi::PyDescr_NewMethod(ty.as_type_ptr(), self.0))
        }
    }

    #[inline]
    fn into_box(self, _py: Python<'p>) -> Box<TypeMember<'p, T> + 'p> {
        Box::new(self)
    }
}


/// Creates a Python class method descriptor that invokes a Rust function.
///
/// As arguments, takes the name of a rust function with the signature
/// `for<'p> fn(&PyType<'p>, &PyTuple<'p>) -> PyResult<'p, T>`
/// for some `T` that implements `ToPyObject`.
///
/// Returns a type that implements `typebuilder::TypeMember<PyRustObject<_>>`
/// by producing an class method descriptor.
///
/// # Example
/// ```
/// #![feature(plugin)]
/// #![plugin(interpolate_idents)]
/// #[macro_use] extern crate cpython;
/// use cpython::{Python, PythonObject, PyResult, PyErr, ObjectProtocol,
///               PyTuple, PyType, PyRustTypeBuilder, NoArgs};
/// use cpython::{exc};
///
/// fn method<'p>(ty: &PyType<'p>, args: &PyTuple<'p>) -> PyResult<'p, i32> {
///     Ok(42)
/// }
///
/// fn main() {
///     let gil = Python::acquire_gil();
///     let my_type = PyRustTypeBuilder::<i32>::new(gil.python(), "MyType")
///       .add("method", py_class_method!(method))
///       .finish().unwrap();
///     let result = my_type.as_object().call_method("method", &NoArgs, None).unwrap();
///     assert_eq!(42, result.extract::<i32>().unwrap());
/// }
/// ```
#[macro_export]
macro_rules! py_class_method {
    ($f: ident) => ( interpolate_idents! {{
        unsafe extern "C" fn [ wrap_ $f ](
            slf: *mut $crate::_detail::ffi::PyObject,
            args: *mut $crate::_detail::ffi::PyObject)
        -> *mut $crate::_detail::ffi::PyObject
        {
            let _guard = $crate::_detail::PanicGuard::with_message("Rust panic in py_method!");
            let py = $crate::_detail::bounded_assume_gil_acquired(&args);
            let slf = $crate::PyObject::from_borrowed_ptr(py, slf);
            let slf = <$crate::PyType as $crate::PythonObject>::unchecked_downcast_from(slf);
            let args = $crate::PyObject::from_borrowed_ptr(py, args);
            let args = <$crate::PyTuple as $crate::PythonObject>::unchecked_downcast_from(args);
            match $f(&slf, &args) {
                Ok(val) => {
                    let obj = $crate::ToPyObject::into_py_object(val, py);
                    return $crate::PythonObject::into_object(obj).steal_ptr();
                }
                Err(e) => {
                    e.restore();
                    return ::std::ptr::null_mut();
                }
            }
        }
        static mut [ method_def_ $f ]: $crate::_detail::ffi::PyMethodDef = $crate::_detail::ffi::PyMethodDef {
            //ml_name: bytes!(stringify!($f), "\0"),
            ml_name: b"<rust method>\0" as *const u8 as *const $crate::_detail::libc::c_char,
            ml_meth: Some([ wrap_ $f ]),
            ml_flags: $crate::_detail::ffi::METH_VARARGS | $crate::_detail::ffi::METH_CLASS,
            ml_doc: 0 as *const $crate::_detail::libc::c_char
        };
        unsafe { $crate::_detail::py_class_method_impl(&mut [ method_def_ $f ], $f) }
    }})
}

pub struct ClassMethodDescriptor(*mut ffi::PyMethodDef);

// py_method_impl takes fn(&T) to ensure that the T in MethodDescriptor<T>
// corresponds to the T in the function signature.
pub unsafe fn py_class_method_impl<'p, R>(
    def: *mut ffi::PyMethodDef,
    _f: fn(&PyType<'p>, &PyTuple<'p>) -> err::PyResult<'p, R>
) -> ClassMethodDescriptor
{
    ClassMethodDescriptor(def)
}

impl <'p, T> TypeMember<'p, T> for ClassMethodDescriptor where T: PythonObject<'p> {
    #[inline]
    fn to_descriptor(&self, ty: &PyType<'p>, _name: &str) -> PyObject<'p> {
        unsafe {
            err::from_owned_ptr_or_panic(ty.python(),
                ffi::PyDescr_NewClassMethod(ty.as_type_ptr(), self.0))
        }
    }

    #[inline]
    fn into_box(self, _py: Python<'p>) -> Box<TypeMember<'p, T> + 'p> {
        Box::new(self)
    }
}

