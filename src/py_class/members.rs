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

use std::marker;
use python::{Python, PythonObject};
use conversion::ToPyObject;
use objects::PyObject;
use err::{self, PyResult};
use ffi;

/// Represents something that can be added as a member to a Python class/type.
///
/// T: type of rust class used for instances of the Python class/type.
pub trait TypeMember<T> where T: PythonObject {
    /// Convert the type member into a python object
    /// that can be stored in the type dict.
    unsafe fn into_descriptor(self, py: Python, ty: *mut ffi::PyTypeObject) -> PyResult<PyObject>;
}

impl <T, S> TypeMember<T> for S where T: PythonObject, S: ToPyObject {
    #[inline]
    unsafe fn into_descriptor(self, py: Python, _ty: *mut ffi::PyTypeObject) -> PyResult<PyObject> {
        Ok(self.into_py_object(py).into_object())
    }
}

#[macro_export]
#[doc(hidden)]
macro_rules! py_class_init_members {
    ($class:ident, $py:ident, $type_object: ident, { }) => {{}};
    ($class:ident, $py:ident, $type_object: ident, { $( $name:ident = $init:expr; )+ }) => {{
        let dict = $crate::PyDict::new($py);
        $( {
            // keep $init out of unsafe block; it might contain user code
            let init = $init;
            let descriptor = try!(unsafe {
                $crate::py_class::members::TypeMember::<$class>::into_descriptor(init, $py, &mut $type_object)
            });
            try!(dict.set_item($py, stringify!($name), descriptor));
        })*
        unsafe {
            assert!($type_object.tp_dict.is_null());
            $type_object.tp_dict = $crate::PythonObject::into_object(dict).steal_ptr();
        }
    }};
}

#[macro_export]
#[doc(hidden)]
macro_rules! py_class_instance_method {
    ($py:ident, $class:ident :: $f:ident [ $( { $pname:ident : $ptype:ty = $detail:tt } )* ]) => {{
        unsafe extern "C" fn wrap_instance_method<DUMMY>(
            slf: *mut $crate::_detail::ffi::PyObject,
            args: *mut $crate::_detail::ffi::PyObject,
            kwargs: *mut $crate::_detail::ffi::PyObject)
        -> *mut $crate::_detail::ffi::PyObject
        {
            const LOCATION: &'static str = concat!(stringify!($class), ".", stringify!($f), "()");
            $crate::_detail::handle_callback(
                LOCATION,
                |py| {
                    py_argparse_raw!(py, Some(LOCATION), args, kwargs,
                        [ $( { $pname : $ptype = $detail } )* ]
                        {
                            let slf = $crate::PyObject::from_borrowed_ptr(py, slf).unchecked_cast_into::<$class>();
                            let ret = slf.$f(py $(, $pname )* );
                            $crate::PyDrop::release_ref(slf, py);
                            ret
                        })
                })
        }
        unsafe {
            let method_def = py_method_def!(stringify!($f), 0, wrap_instance_method::<()>);
            $crate::py_class::members::create_instance_method_descriptor(method_def)
        }
    }}
}

pub struct InstanceMethodDescriptor<T>(*mut ffi::PyMethodDef, marker::PhantomData<fn(&T)>);

#[inline]
pub unsafe fn create_instance_method_descriptor<T>(method_def: *mut ffi::PyMethodDef)
  -> InstanceMethodDescriptor<T>
{
    InstanceMethodDescriptor(method_def, marker::PhantomData)
}

impl <T> TypeMember<T> for InstanceMethodDescriptor<T> where T: PythonObject {
    #[inline]
    unsafe fn into_descriptor(self, py: Python, ty: *mut ffi::PyTypeObject) -> PyResult<PyObject> {
        err::result_from_owned_ptr(py, ffi::PyDescr_NewMethod(ty, self.0))
    }
}

