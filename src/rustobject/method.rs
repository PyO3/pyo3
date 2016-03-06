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

use std::{marker, mem};
use python::{Python, PythonObject};
use objects::{PyObject, PyTuple, PyType};
use super::typebuilder;
use ffi;
use err;

#[macro_export]
#[doc(hidden)]
macro_rules! py_method_wrap {
    // * $f: function name, used as part of wrapper function name
    // * |py, slf, args, kwargs| { body }
    ($f: ident, | $py: ident, $slf: ident, $args: ident, $kwargs: ident | $body: block) => {{
        unsafe extern "C" fn wrap<DUMMY>(
            slf: *mut $crate::_detail::ffi::PyObject,
            args: *mut $crate::_detail::ffi::PyObject,
            kwargs: *mut $crate::_detail::ffi::PyObject)
        -> *mut $crate::_detail::ffi::PyObject
        {
            let _guard = $crate::_detail::PanicGuard::with_message(
                concat!("Rust panic in py_method!(", stringify!($f), ")"));
            let $py: $crate::Python = $crate::_detail::bounded_assume_gil_acquired(&args);
            let slf: $crate::PyObject = $crate::PyObject::from_borrowed_ptr($py, slf);
            let args: $crate::PyTuple = $crate::PyObject::from_borrowed_ptr($py, args).unchecked_cast_into();
            let kwargs: Option<$crate::PyDict> = $crate::_detail::get_kwargs($py, kwargs);
            let ret = {
                let $slf = &slf;
                let $args = &args;
                let $kwargs = kwargs.as_ref();
                $crate::_detail::result_to_ptr($py, $body)
            };
            $crate::PyDrop::release_ref(slf, $py);
            $crate::PyDrop::release_ref(args, $py);
            $crate::PyDrop::release_ref(kwargs, $py);
            ret
        }
        wrap::<()>
    }};
}

/// Creates a Python instance method descriptor that invokes a Rust function.

/// There are two forms of this macro:
/// 1) py_method!(f)
///     `f` is the name of a rust function with the signature
///     `fn(Python, &SelfType, &PyTuple, Option<&PyDict>) -> PyResult<R>`
///      for some `R` that implements `ToPyObject`.
///
/// 2) py_method!(f(parameter_list))
///     This form automatically converts the arguments into
///     the Rust types specified in the parameter list,
///     and then calls `f(Python, &SelfType, Parameters)`.
///     See `py_argparse!()` for details on argument parsing.
///
/// Returns an unspecified type that implements `typebuilder::TypeMember<SelfType>`.
/// When the member is added to a type, it results in an instance method descriptor.
///
/// # Example
/// ```
/// #[macro_use] extern crate cpython;
/// use cpython::{Python, PythonObject, PyResult, PyErr, ObjectProtocol};
/// use cpython::{exc};
/// use cpython::rustobject::{PyRustObject, PyRustTypeBuilder};
///
/// fn mul(py: Python, slf: &PyRustObject<i32>, arg: i32) -> PyResult<i32> {
///     match slf.get(py).checked_mul(arg) {
///         Some(val) => Ok(val),
///         None => Err(PyErr::new_lazy_init(py.get_type::<exc::OverflowError>(), None))
///     }
/// }
///
/// fn main() {
///     let gil = Python::acquire_gil();
///     let py = gil.python();
///     let multiplier_type = PyRustTypeBuilder::<i32>::new(py, "Multiplier")
///       .add("mul", py_method!(mul(arg: i32)))
///       .finish().unwrap();
///     let obj = multiplier_type.create_instance(py, 3, ()).into_object();
///     let result = obj.call_method(py, "mul", &(4,), None).unwrap().extract::<i32>(py).unwrap();
///     assert_eq!(result, 12);
/// }
/// ```
#[macro_export]
macro_rules! py_method {
    ($f: ident) => ({
        let wrap = py_method_wrap!($f, |py, slf, args, kwargs| {
            let slf = slf.unchecked_cast_as();
            $f(py, slf, args, kwargs)
        });
        unsafe {
            $crate::rustobject::py_method_impl::py_method_impl(
                py_method_def!($f, 0, wrap), $f)
        }
    });
    ($f: ident ( $( $pname:ident : $ptype:ty ),* ) ) => ({
        let wrap = py_method_wrap!($f, |py, slf, args, kwargs| {
            let slf = slf.unchecked_cast_as();
            py_argparse!(py, Some(stringify!($f)), args, kwargs,
                ( $($pname : $ptype),* ) { $f( py, slf, $($pname),* ) })
        });
        unsafe {
            py_method_call_impl!(
                py_method_def!($f, 0, wrap),
                $f ( $($pname : $ptype),* ) )
        }
    })
}

/// Return value of `py_method!()` macro.
pub struct MethodDescriptor<T>(*mut ffi::PyMethodDef, marker::PhantomData<fn(&T)>);

#[doc(hidden)]
pub mod py_method_impl {
    use ffi;
    use err;
    use python::Python;
    use objects::{PyTuple, PyDict};
    use super::MethodDescriptor;
    use std::marker;

    // py_method_impl takes fn(&T) to ensure that the T in MethodDescriptor<T>
    // corresponds to the T in the function signature.
    pub unsafe fn py_method_impl<T, R>(
        def: *mut ffi::PyMethodDef,
        _f: fn(Python, &T, &PyTuple, Option<&PyDict>) -> err::PyResult<R>
    ) -> MethodDescriptor<T> {
        MethodDescriptor(def, marker::PhantomData)
    }

    #[macro_export]
    #[doc(hidden)]
    macro_rules! py_method_call_impl {
        ( $def:expr, $f:ident ( ) )
            => { $crate::rustobject::py_method_impl::py_method_impl_0($def, $f) };
        ( $def:expr, $f:ident ( $n1:ident : $t1:ty ) )
            => { $crate::rustobject::py_method_impl::py_method_impl_1($def, $f) };
        ( $def:expr, $f:ident ( $n1:ident : $t1:ty, $n2:ident : $t2:ty ) )
            => { $crate::rustobject::py_method_impl::py_method_impl_2($def, $f) };
        ( $def:expr, $f:ident ( $n1:ident : $t1:ty, $n2:ident : $t2:ty, $n3:ident : $t3:ty ) )
            => { $crate::rustobject::py_method_impl::py_method_impl_3($def, $f) };
        ( $def:expr, $f:ident ( $n1:ident : $t1:ty, $n2:ident : $t2:ty, $n3:ident : $t3:ty, $n4:ident : $t4:ty ) )
            => { $crate::rustobject::py_method_impl::py_method_impl_4($def, $f) };
    }

    pub unsafe fn py_method_impl_0<T, R>(
        def: *mut ffi::PyMethodDef,
        _f: fn(Python, &T) -> err::PyResult<R>
    ) -> MethodDescriptor<T> {
        MethodDescriptor(def, marker::PhantomData)
    }

    pub unsafe fn py_method_impl_1<T, P1, R>(
        def: *mut ffi::PyMethodDef,
        _f: fn(Python, &T, P1) -> err::PyResult<R>
    ) -> MethodDescriptor<T> {
        MethodDescriptor(def, marker::PhantomData)
    }

    pub unsafe fn py_method_impl_2<T, P1, P2, R>(
        def: *mut ffi::PyMethodDef,
        _f: fn(Python, &T, P1, P2) -> err::PyResult<R>
    ) -> MethodDescriptor<T> {
        MethodDescriptor(def, marker::PhantomData)
    }

    pub unsafe fn py_method_impl_3<T, P1, P2, P3, R>(
        def: *mut ffi::PyMethodDef,
        _f: fn(Python, &T, P1, P2, P3) -> err::PyResult<R>
    ) -> MethodDescriptor<T> {
        MethodDescriptor(def, marker::PhantomData)
    }

    pub unsafe fn py_method_impl_4<T, P1, P2, P3, P4, R>(
        def: *mut ffi::PyMethodDef,
        _f: fn(Python, &T, P1, P2, P3, P4) -> err::PyResult<R>
    ) -> MethodDescriptor<T> {
        MethodDescriptor(def, marker::PhantomData)
    }
}

impl <T> typebuilder::TypeMember<T> for MethodDescriptor<T> where T: PythonObject {
    #[inline]
    fn to_descriptor(&self, py: Python, ty: &PyType, _name: &str) -> PyObject {
        unsafe {
            err::from_owned_ptr_or_panic(py,
                ffi::PyDescr_NewMethod(ty.as_type_ptr(), self.0))
        }
    }
}

#[macro_export]
#[doc(hidden)]
macro_rules! py_class_method_wrap {
    // * $f: function name, used as part of wrapper function name
    // * |py, cls, args, kwargs| { body }
    ($f: ident, | $py: ident, $slf: ident, $args: ident, $kwargs: ident | $body: block) => {{
        unsafe extern "C" fn wrap<DUMMY>(
            slf: *mut $crate::_detail::ffi::PyObject,
            args: *mut $crate::_detail::ffi::PyObject,
            kwargs: *mut $crate::_detail::ffi::PyObject)
        -> *mut $crate::_detail::ffi::PyObject
        {
            let _guard = $crate::_detail::PanicGuard::with_message(
                concat!("Rust panic in py_class_method!(", stringify!($f), ")"));
            let $py: $crate::Python = $crate::_detail::bounded_assume_gil_acquired(&args);
            let slf: $crate::PyType = $crate::PyObject::from_borrowed_ptr($py, slf).unchecked_cast_into();
            let args: $crate::PyTuple = $crate::PyObject::from_borrowed_ptr($py, args).unchecked_cast_into();
            let kwargs: Option<$crate::PyDict> = $crate::_detail::get_kwargs($py, kwargs);
            let ret = {
                let $slf = &slf;
                let $args = &args;
                let $kwargs = kwargs.as_ref();
                $crate::_detail::result_to_ptr($py, $body)
            };
            $crate::PyDrop::release_ref(slf, $py);
            $crate::PyDrop::release_ref(args, $py);
            $crate::PyDrop::release_ref(kwargs, $py);
            ret
        }
        wrap::<()>
    }};
}

/// Creates a Python class method descriptor that invokes a Rust function.
///
/// There are two forms of this macro:
/// 1) py_class_method!(f)
///     `f` is the name of a rust function with the signature
///     `fn(Python, &PyType, &PyTuple, Option<&PyDict>) -> PyResult<R>`
///      for some `R` that implements `ToPyObject`.
///
/// 2) py_class_method!(f(parameter_list))
///     This form automatically converts the arguments into
///     the Rust types specified in the parameter list,
///     and then calls `f(Python, &PyType, Parameters)`.
///     See `py_argparse!()` for details on argument parsing.
///
/// Returns a type that implements `typebuilder::TypeMember`
/// by producing an class method descriptor.
///
/// # Example
/// ```
/// #[macro_use] extern crate cpython;
/// use cpython::{Python, PythonObject, PyResult, ObjectProtocol, PyType, NoArgs};
/// use cpython::rustobject::PyRustTypeBuilder;
///
/// fn method(py: Python, cls: &PyType) -> PyResult<i32> {
///     Ok(42)
/// }
///
/// fn main() {
///     let gil = Python::acquire_gil();
///     let py = gil.python();
///     let my_type = PyRustTypeBuilder::<i32>::new(py, "MyType")
///       .add("method", py_class_method!(method()))
///       .finish().unwrap();
///     let result = my_type.as_object().call_method(py, "method", NoArgs, None).unwrap();
///     assert_eq!(42, result.extract::<i32>(py).unwrap());
/// }
/// ```
#[macro_export]
macro_rules! py_class_method {
    ($f: ident) => ({
        let wrap = py_class_method_wrap!($f, |py, cls, args, kwargs| {
            $f(py, cls, args, kwargs)
        });
        unsafe {
            $crate::rustobject::py_class_method_impl(
                py_method_def!($f, $crate::_detail::ffi::METH_CLASS, wrap))
        }
    });
    ($f: ident ( $( $pname:ident : $ptype:ty ),* ) ) => ({
        let wrap = py_class_method_wrap!($f, |py, cls, args, kwargs| {
            py_argparse!(py, Some(stringify!($f)), args, kwargs,
                    ( $($pname : $ptype),* ) { $f( py, cls, $($pname),* ) })
        });
        unsafe {
            $crate::rustobject::py_class_method_impl(
                py_method_def!($f, $crate::_detail::ffi::METH_CLASS, wrap))
        }
    });
}

/// The return type of the `py_class_method!()` macro.
pub struct ClassMethodDescriptor(*mut ffi::PyMethodDef);

#[inline]
#[doc(hidden)]
pub unsafe fn py_class_method_impl(def: *mut ffi::PyMethodDef) -> ClassMethodDescriptor {
    ClassMethodDescriptor(def)
}

impl <T> typebuilder::TypeMember<T> for ClassMethodDescriptor where T: PythonObject {
    #[inline]
    fn to_descriptor(&self, py: Python, ty: &PyType, _name: &str) -> PyObject {
        unsafe {
            err::from_owned_ptr_or_panic(py,
                ffi::PyDescr_NewClassMethod(ty.as_type_ptr(), self.0))
        }
    }
}

unsafe impl typebuilder::TypeConstructor for ClassMethodDescriptor {
    fn tp_new(&self) -> ffi::newfunc {
        unsafe {
            mem::transmute::<ffi::PyCFunction, ffi::newfunc>((*self.0).ml_meth.unwrap())
        }
    }
}

