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

use ffi;
use std::{mem, isize, ptr};
use libc::c_int;
use python::{Python, PythonObject};
use conversion::ToPyObject;
use function::CallbackConverter;
use err::{PyErr};
use exc;
use Py_hash_t;

#[macro_export]
#[doc(hidden)]
macro_rules! py_class_type_object_static_init {
    ($class_name:ident,
     $gc:tt,
    /* slots: */ {
        /* type_slots */  [ $( $slot_name:ident : $slot_value:expr, )* ]
        $as_number:tt
        $as_sequence:tt
        $as_mapping:tt
    }) => (
        $crate::_detail::ffi::PyTypeObject {
            $( $slot_name : $slot_value, )*
            tp_dealloc: Some($crate::py_class::slots::tp_dealloc_callback::<$class_name>),
            tp_flags: py_class_type_object_flags!($gc),
            tp_traverse: py_class_tp_traverse!($class_name, $gc),
            ..
            $crate::_detail::ffi::PyTypeObject_INIT
        }
    );
}

#[macro_export]
#[doc(hidden)]
macro_rules! py_class_type_object_flags {
    (/* gc: */ {
        /* traverse_proc: */ None,
        /* traverse_data: */ [ /*name*/ ]
    }) => {
        $crate::_detail::ffi::Py_TPFLAGS_DEFAULT
    };
    (/* gc: */ {
        $traverse_proc: expr,
        $traverse_data: tt
    }) => {
        $crate::_detail::ffi::Py_TPFLAGS_DEFAULT
        | $crate::_detail::ffi::Py_TPFLAGS_HAVE_GC
    };
}

#[macro_export]
#[doc(hidden)]
macro_rules! py_class_type_object_dynamic_init {
    // initialize those fields of PyTypeObject that we couldn't initialize statically
    ($class: ident, $py:ident, $type_object:ident,
        /* slots: */ {
            $type_slots:tt
            $as_number:tt
            $as_sequence:tt
            $as_mapping:tt
        }
    ) => {
        unsafe {
            $type_object.tp_name = concat!(stringify!($class), "\0").as_ptr() as *const _;
            $type_object.tp_basicsize = <$class as $crate::py_class::BaseObject>::size()
                                        as $crate::_detail::ffi::Py_ssize_t;
        }
        // call slot macros outside of unsafe block
        *(unsafe { &mut $type_object.tp_as_sequence }) = py_class_as_sequence!($as_sequence);
        *(unsafe { &mut $type_object.tp_as_number }) = py_class_as_number!($as_number);
        *(unsafe { &mut $type_object.tp_as_mapping }) = py_class_as_mapping!($as_mapping);
    }
}

pub unsafe extern "C" fn tp_dealloc_callback<T>(obj: *mut ffi::PyObject)
    where T: super::BaseObject
{
    let guard = ::function::AbortOnDrop("Cannot unwind out of tp_dealloc");
    let py = Python::assume_gil_acquired();
    let r = T::dealloc(py, obj);
    mem::forget(guard);
    r
}

#[macro_export]
#[doc(hidden)]
macro_rules! py_class_wrap_newfunc {
    ($class:ident :: $f:ident [ $( { $pname:ident : $ptype:ty = $detail:tt } )* ]) => {{
        unsafe extern "C" fn wrap_newfunc<DUMMY>(
            cls: *mut $crate::_detail::ffi::PyTypeObject,
            args: *mut $crate::_detail::ffi::PyObject,
            kwargs: *mut $crate::_detail::ffi::PyObject)
        -> *mut $crate::_detail::ffi::PyObject
        {
            const LOCATION: &'static str = concat!(stringify!($class), ".", stringify!($f), "()");
            $crate::_detail::handle_callback(
                LOCATION, $crate::_detail::PyObjectCallbackConverter,
                |py| {
                    py_argparse_raw!(py, Some(LOCATION), args, kwargs,
                        [ $( { $pname : $ptype = $detail } )* ]
                        {
                            let cls = $crate::PyType::from_type_ptr(py, cls);
                            let ret = $class::$f(&cls, py $(, $pname )* );
                            $crate::PyDrop::release_ref(cls, py);
                            ret
                        })
                })
        }
        Some(wrap_newfunc::<()>)
    }}
}


#[macro_export]
#[doc(hidden)]
macro_rules! py_class_as_sequence {
    ([]) => (0 as *mut $crate::_detail::ffi::PySequenceMethods);
    ([$( $slot_name:ident : $slot_value:expr ,)+]) => {{
        static mut SEQUENCE_METHODS : $crate::_detail::ffi::PySequenceMethods
            = $crate::_detail::ffi::PySequenceMethods {
                $( $slot_name : $slot_value, )*
                ..
                $crate::_detail::ffi::PySequenceMethods_INIT
            };
        unsafe { &mut SEQUENCE_METHODS }
    }}
}

#[macro_export]
#[doc(hidden)]
macro_rules! py_class_as_number {
    ([]) => (0 as *mut $crate::_detail::ffi::PyNumberMethods);
    ([$( $slot_name:ident : $slot_value:expr ,)+]) => {{
        static mut NUMBER_METHODS : $crate::_detail::ffi::PyNumberMethods
            = $crate::_detail::ffi::PyNumberMethods {
                $( $slot_name : $slot_value, )*
                ..
                $crate::_detail::ffi::PyNumberMethods_INIT
            };
        unsafe { &mut NUMBER_METHODS }
    }}
}

#[macro_export]
#[doc(hidden)]
macro_rules! py_class_as_mapping {
    ([]) => (0 as *mut $crate::_detail::ffi::PyMappingMethods);
    ([$( $slot_name:ident : $slot_value:expr ,)+]) => {{
        static mut MAPPING_METHODS : $crate::_detail::ffi::PyMappingMethods
            = $crate::_detail::ffi::PyMappingMethods {
                $( $slot_name : $slot_value, )*
                ..
                $crate::_detail::ffi::PyMappingMethods_INIT
            };
        unsafe { &mut MAPPING_METHODS }
    }}
}


#[macro_export]
#[doc(hidden)]
macro_rules! py_class_unary_slot {
    ($class:ident :: $f:ident, $res_type:ty, $conv:expr) => {{
        unsafe extern "C" fn wrap_unary<DUMMY>(
            slf: *mut $crate::_detail::ffi::PyObject)
        -> $res_type
        {
            const LOCATION: &'static str = concat!(stringify!($class), ".", stringify!($f), "()");
            $crate::_detail::handle_callback(
                LOCATION, $conv,
                |py| {
                    let slf = $crate::PyObject::from_borrowed_ptr(py, slf).unchecked_cast_into::<$class>();
                    let ret = slf.$f(py);
                    $crate::PyDrop::release_ref(slf, py);
                    ret
                })
        }
        Some(wrap_unary::<()>)
    }}
}

#[macro_export]
#[doc(hidden)]
macro_rules! py_class_binary_slot {
    ($class:ident :: $f:ident, $arg_type:ty, $res_type:ty, $conv:expr) => {{
        unsafe extern "C" fn wrap_unary<DUMMY>(
            slf: *mut $crate::_detail::ffi::PyObject,
            arg: *mut $crate::_detail::ffi::PyObject)
        -> $res_type
        {
            const LOCATION: &'static str = concat!(stringify!($class), ".", stringify!($f), "()");
            $crate::_detail::handle_callback(
                LOCATION, $conv,
                |py| {
                    let slf = $crate::PyObject::from_borrowed_ptr(py, slf).unchecked_cast_into::<$class>();
                    let arg = $crate::PyObject::from_borrowed_ptr(py, arg);
                    let ret = match <$arg_type as $crate::ExtractPyObject>::prepare_extract(py, &arg) {
                        Ok(prepared) => {
                            match <$arg_type as $crate::ExtractPyObject>::extract(py, &prepared) {
                                Ok(arg) => slf.$f(py, arg),
                                Err(e) => Err(e)
                            }
                        },
                        Err(e) => Err(e)
                    };
                    $crate::PyDrop::release_ref(arg, py);
                    $crate::PyDrop::release_ref(slf, py);
                    ret
                })
        }
        Some(wrap_unary::<()>)
    }}
}

pub struct LenResultConverter;

impl CallbackConverter<usize> for LenResultConverter {
    type R = isize;

    fn convert(val: usize, py: Python) -> isize {
        if val <= (isize::MAX as usize) {
            val as isize
        } else {
            PyErr::new_lazy_init(py.get_type::<exc::OverflowError>(), None).restore(py);
            -1
        }
    }

    fn error_value() -> isize {
        -1
    }
}

pub struct IterNextResultConverter;

impl <T> CallbackConverter<Option<T>>
    for IterNextResultConverter
    where T: ToPyObject
{
    type R = *mut ffi::PyObject;

    fn convert(val: Option<T>, py: Python) -> *mut ffi::PyObject {
        match val {
            Some(val) => val.into_py_object(py).into_object().steal_ptr(),
            None => unsafe {
                ffi::PyErr_SetNone(ffi::PyExc_StopIteration);
                ptr::null_mut()
            }
        }
    }

    fn error_value() -> *mut ffi::PyObject {
        ptr::null_mut()
    }
}

pub trait WrappingCastTo<T> {
    fn wrapping_cast(self) -> T;
}

macro_rules! wrapping_cast {
    ($from:ty, $to:ty) => {
        impl WrappingCastTo<$to> for $from {
            #[inline]
            fn wrapping_cast(self) -> $to {
                self as $to
            }
        }
    }
}
wrapping_cast!(u8, Py_hash_t);
wrapping_cast!(u16, Py_hash_t);
wrapping_cast!(u32, Py_hash_t);
wrapping_cast!(usize, Py_hash_t);
wrapping_cast!(u64, Py_hash_t);
wrapping_cast!(i8, Py_hash_t);
wrapping_cast!(i16, Py_hash_t);
wrapping_cast!(i32, Py_hash_t);
wrapping_cast!(isize, Py_hash_t);
wrapping_cast!(i64, Py_hash_t);

pub struct HashConverter;

impl <T> CallbackConverter<T> for HashConverter
    where T: WrappingCastTo<Py_hash_t>
{
    type R = Py_hash_t;

    #[inline]
    fn convert(val: T, _py: Python) -> Py_hash_t {
        let hash = val.wrapping_cast();
        if hash == -1 {
            -2
        } else {
            hash
        }
    }

    #[inline]
    fn error_value() -> Py_hash_t {
        -1
    }
}

pub struct BoolConverter;

impl CallbackConverter<bool> for BoolConverter {
    type R = c_int;

    #[inline]
    fn convert(val: bool, _py: Python) -> c_int {
        val as c_int
    }

    #[inline]
    fn error_value() -> c_int {
        -1
    }
}


#[macro_export]
#[doc(hidden)]
macro_rules! py_class_call_slot {
    ($class:ident :: $f:ident [ $( { $pname:ident : $ptype:ty = $detail:tt } )* ]) => {{
        unsafe extern "C" fn wrap_call<DUMMY>(
            slf: *mut $crate::_detail::ffi::PyObject,
            args: *mut $crate::_detail::ffi::PyObject,
            kwargs: *mut $crate::_detail::ffi::PyObject)
        -> *mut $crate::_detail::ffi::PyObject
        {
            const LOCATION: &'static str = concat!(stringify!($class), ".", stringify!($f), "()");
            $crate::_detail::handle_callback(
                LOCATION, $crate::_detail::PyObjectCallbackConverter,
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
        Some(wrap_call::<()>)
    }}
}

/// Used as implementation in the `sq_item` slot to forward calls to the `mp_subscript` slot.
pub unsafe extern "C" fn sq_item(obj: *mut ffi::PyObject, index: ffi::Py_ssize_t) -> *mut ffi::PyObject {
    let arg = ffi::PyLong_FromSsize_t(index);
    if arg.is_null() {
        return arg;
    }
    let ret = ffi::PyObject_GetItem(obj, arg);
    ffi::Py_DECREF(arg);
    ret
}

